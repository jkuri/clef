use crate::config::AppConfig;
use crate::error::ApiError;
use crate::models::Package;
use crate::state::AppState;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use rocket::serde::json::Value;

pub struct RegistryService;

impl RegistryService {
    fn rewrite_tarball_urls(
        json: &mut Value,
        config: &AppConfig,
        scheme: &str,
        request_host: Option<&str>,
    ) -> Result<(), ApiError> {
        // Rewrite tarball URLs in package metadata to point to our proxy server
        if let Some(versions) = json.get_mut("versions").and_then(|v| v.as_object_mut()) {
            for (version, version_data) in versions.iter_mut() {
                if let Some(dist) = version_data.get_mut("dist").and_then(|d| d.as_object_mut()) {
                    if let Some(tarball_url) = dist
                        .get("tarball")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                    {
                        // Extract package name and filename from the original tarball URL
                        // Use the configured upstream registry instead of hardcoded URL
                        if tarball_url.starts_with(&config.upstream_registry) {
                            if let Some(path_part) =
                                tarball_url.strip_prefix(&format!("{}/", config.upstream_registry))
                            {
                                // Use request host if available, otherwise fall back to config host
                                let host_to_use = request_host.unwrap_or(&config.host);

                                // Rewrite to our proxy server URL using the same scheme as the request
                                let new_url =
                                    format!("{scheme}://{host_to_use}/registry/{path_part}");

                                dist.insert("tarball".to_string(), Value::String(new_url.clone()));
                                debug!(
                                    "Rewrote tarball URL for {version}: {tarball_url} -> {new_url}"
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn store_package_metadata_in_database(
        package: &str,
        json: &Value,
        state: &AppState,
    ) -> Result<(), ApiError> {
        // Extract basic package information from the npm metadata
        let description = json["description"].as_str().map(|s| s.to_string());

        // Create or get the package
        let pkg = state
            .database
            .create_or_get_package(package, description, None)
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

        // Extract and store version information from the npm registry response
        if let Some(versions) = json["versions"].as_object() {
            for (version_str, version_data) in versions {
                // Store version with full metadata from npm registry
                // The create_or_get_package_version_with_metadata method will handle existing versions
                if let Err(e) = state.database.create_or_get_package_version_with_metadata(
                    pkg.id,
                    version_str,
                    version_data,
                ) {
                    warn!("Failed to store version metadata for {package}/{version_str}: {e}");
                } else {
                    debug!("Stored version metadata for {package}/{version_str}");
                }
            }
        }

        Ok(())
    }

    async fn store_version_metadata_in_database(
        package: &str,
        version: &str,
        json: &Value,
        state: &AppState,
    ) -> Result<(), ApiError> {
        // Extract basic package information
        let description = json["description"].as_str().map(|s| s.to_string());

        // Create or get the package
        let pkg = state
            .database
            .create_or_get_package(package, description, None)
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

        // Store the specific version with metadata
        if let Err(e) = state
            .database
            .create_or_get_package_version_with_metadata(pkg.id, version, json)
        {
            warn!("Failed to store version metadata for {package}/{version}: {e}");
        } else {
            debug!("Stored version metadata for {package}/{version}");
        }

        Ok(())
    }

    pub async fn get_package_metadata(
        package: &str,
        state: &AppState,
        request_host: Option<&str>,
        request_scheme: &str,
    ) -> Result<Value, ApiError> {
        info!("Fetching metadata for package: {package}");

        // Check metadata cache first
        if let Some(cache_entry) = state.cache.get_metadata(package).await {
            info!(
                "Metadata cache hit for package: {} (size: {} bytes)",
                package,
                cache_entry.data.len()
            );
            let metadata_str = String::from_utf8(cache_entry.data).map_err(|e| {
                ApiError::InternalServerError(format!("Invalid UTF-8 in cached metadata: {e}"))
            })?;
            let metadata: Value = serde_json::from_str(&metadata_str).map_err(|e| {
                ApiError::InternalServerError(format!("Invalid JSON in cached metadata: {e}"))
            })?;
            return Ok(metadata);
        }

        info!("Metadata cache miss for package: {package}, generating fresh metadata");

        // First check if we have any published versions of this package in our database
        let mut conn = state.database.get_connection().map_err(|e| {
            ApiError::InternalServerError(format!("Database connection error: {e}"))
        })?;

        use crate::schema::packages;
        let published_packages: Vec<Package> = packages::table
            .filter(packages::name.eq(package))
            .filter(packages::author_id.is_not_null()) // Only published packages have author_id
            .load::<Package>(&mut conn)
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

        let metadata = if !published_packages.is_empty() {
            // We have published versions, generate metadata from our database
            info!(
                "Found {} published versions for package: {}",
                published_packages.len(),
                package
            );
            Self::generate_metadata_from_published_packages(
                package,
                &published_packages,
                state,
                request_host,
                request_scheme,
            )?
        } else {
            // No published versions found, proxy to upstream
            let url = format!("{}/{package}", state.config.upstream_registry);

            // Check if we have cached metadata with ETag for conditional request
            let mut request = state.client.get(&url);

            // Add If-None-Match header if we have cached ETag
            if let Some(cache_entry) = state.cache.get_metadata(package).await {
                if let Some(etag) = &cache_entry.etag {
                    debug!("Adding If-None-Match header for upstream request: {etag}");
                    request = request.header("If-None-Match", etag);
                }
            }

            let response = request.send().await?;

            if response.status() == 304 {
                // Not Modified - use cached version
                debug!("Upstream returned 304 Not Modified for package: {package}");
                if let Some(cache_entry) = state.cache.get_metadata(package).await {
                    info!(
                        "Using cached metadata after 304 Not Modified for package: {package} (size: {} bytes)",
                        cache_entry.data.len()
                    );
                    let metadata_str = String::from_utf8(cache_entry.data).map_err(|e| {
                        ApiError::InternalServerError(format!(
                            "Invalid UTF-8 in cached metadata: {e}"
                        ))
                    })?;
                    let metadata: Value = serde_json::from_str(&metadata_str).map_err(|e| {
                        ApiError::InternalServerError(format!(
                            "Invalid JSON in cached metadata: {e}"
                        ))
                    })?;
                    return Ok(metadata);
                } else {
                    return Err(ApiError::InternalServerError(
                        "Received 304 but no cached metadata found".to_string(),
                    ));
                }
            } else if response.status().is_success() {
                // Extract ETag for future conditional requests
                let etag = response
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                match response.json::<Value>().await {
                    Ok(mut json) => {
                        // Rewrite tarball URLs to point to our proxy server
                        Self::rewrite_tarball_urls(
                            &mut json,
                            &state.config,
                            request_scheme,
                            request_host,
                        )?;

                        info!("Successfully proxied metadata for package: {package}");

                        // Store basic package information in database for analytics
                        if let Err(e) =
                            Self::store_package_metadata_in_database(package, &json, state).await
                        {
                            warn!("Failed to store package metadata in database: {e:?}");
                        }

                        // Cache with ETag if available
                        let metadata_str = serde_json::to_string(&json).map_err(|e| {
                            ApiError::InternalServerError(format!(
                                "Failed to serialize metadata for caching: {e}"
                            ))
                        })?;

                        if let Err(e) = state
                            .cache
                            .put_metadata_with_etag(package, &metadata_str, etag.as_deref())
                            .await
                        {
                            warn!("Failed to cache metadata for package {package}: {e}");
                        }

                        return Ok(json);
                    }
                    Err(e) => {
                        error!("Failed to parse JSON response for package {package}: {e}");
                        return Err(ApiError::ParseError(format!(
                            "Failed to parse upstream response: {e}"
                        )));
                    }
                }
            } else {
                error!(
                    "Upstream returned error {} for package: {package}",
                    response.status()
                );
                return Err(ApiError::UpstreamError(format!(
                    "Upstream error: {}",
                    response.status()
                )));
            }
        };

        // Cache the metadata
        let metadata_str = serde_json::to_string(&metadata).map_err(|e| {
            ApiError::InternalServerError(format!("Failed to serialize metadata for caching: {e}"))
        })?;

        if let Err(e) = state.cache.put_metadata(package, &metadata_str).await {
            warn!("Failed to cache metadata for package {package}: {e}");
        }

        Ok(metadata)
    }

    pub async fn get_package_version_metadata(
        package: &str,
        version: &str,
        state: &AppState,
    ) -> Result<Value, ApiError> {
        info!("Fetching metadata for package: {package} version: {version}");

        let url = format!("{}/{package}/{version}", state.config.upstream_registry);

        let response = state.client.get(&url).send().await?;

        if response.status().is_success() {
            match response.json::<Value>().await {
                Ok(json) => {
                    info!(
                        "Successfully proxied metadata for package: {package} version: {version}"
                    );

                    // Store version metadata in database for analytics and future use
                    if let Err(e) =
                        Self::store_version_metadata_in_database(package, version, &json, state)
                            .await
                    {
                        warn!("Failed to store version metadata in database: {e:?}");
                    }

                    Ok(json)
                }
                Err(e) => {
                    error!(
                        "Failed to parse JSON response for package {package} version {version}: {e}"
                    );
                    Err(ApiError::ParseError(format!(
                        "Failed to parse upstream response: {e}"
                    )))
                }
            }
        } else {
            error!(
                "Upstream returned error {} for package: {} version: {}",
                response.status(),
                package,
                version
            );
            Err(ApiError::UpstreamError(format!(
                "Upstream error: {}",
                response.status()
            )))
        }
    }

    pub async fn get_package_tarball(
        package: &str,
        filename: &str,
        state: &AppState,
    ) -> Result<Vec<u8>, ApiError> {
        info!("Fetching tarball for package: {package} filename: {filename}");

        // Check cache first
        if let Some(cache_entry) = state
            .cache
            .get(package, filename, Some(&*state.database))
            .await
        {
            info!(
                "Cache hit for tarball: {package} filename: {filename} (size: {} bytes)",
                cache_entry.data.len()
            );
            return Ok(cache_entry.data);
        }

        // Cache miss, fetch from upstream
        let url = format!(
            "{}/{}/-/{filename}",
            state.config.upstream_registry, package
        );

        let response = state.client.get(&url).send().await?;

        if response.status().is_success() {
            // Extract ETag for cache validation
            let etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match response.bytes().await {
                Ok(bytes) => {
                    let data = bytes.to_vec();

                    // Store in cache
                    if let Err(e) = state
                        .cache
                        .put(
                            package,
                            filename,
                            &data,
                            etag.as_deref(),
                            &url,
                            Some(&*state.database),
                        )
                        .await
                    {
                        error!("Failed to cache tarball for {package} filename {filename}: {e}");
                    }

                    info!(
                        "Successfully proxied and cached tarball for package: {package} filename: {filename} (size: {} bytes)",
                        data.len()
                    );
                    Ok(data)
                }
                Err(e) => {
                    error!(
                        "Failed to read bytes from response for package {package} filename {filename}: {e}"
                    );
                    Err(ApiError::ParseError(format!(
                        "Failed to read upstream response: {e}"
                    )))
                }
            }
        } else {
            error!(
                "Upstream returned error {} for package: {package} filename: {filename}",
                response.status()
            );
            Err(ApiError::UpstreamError(format!(
                "Upstream error: {}",
                response.status()
            )))
        }
    }

    pub async fn head_package_tarball(
        package: &str,
        filename: &str,
        state: &AppState,
    ) -> Result<(), ApiError> {
        info!("HEAD request for tarball: {package} filename: {filename}");

        // Check cache first
        if state
            .cache
            .get(package, filename, Some(&*state.database))
            .await
            .is_some()
        {
            info!("Cache hit for HEAD tarball: {package} filename: {filename}");
            return Ok(());
        }

        // Cache miss, check upstream
        let url = format!(
            "{}/{}/-/{}",
            state.config.upstream_registry, package, filename
        );

        let response = state.client.head(&url).send().await?;

        if response.status().is_success() {
            info!("Successfully checked tarball for package: {package} filename: {filename}");
            Ok(())
        } else {
            error!(
                "Upstream returned error {} for HEAD package: {package} filename: {filename}",
                response.status()
            );
            Err(ApiError::UpstreamError(format!(
                "Upstream error: {}",
                response.status()
            )))
        }
    }

    fn load_package_json_from_filesystem(
        package_name: &str,
        version: &str,
        state: &AppState,
    ) -> Result<Option<Value>, ApiError> {
        use std::path::Path;

        let cache_dir = Path::new(&state.config.cache_dir);
        let packages_dir = cache_dir.join("packages");
        let package_dir = packages_dir.join(package_name);

        // Generate the package.json filename
        let package_json_filename = format!(
            "{}-{}.json",
            if package_name.starts_with('@') {
                package_name.split('/').next_back().unwrap_or(package_name)
            } else {
                package_name
            },
            version
        );
        let package_json_path = package_dir.join(package_json_filename);

        if package_json_path.exists() {
            let package_json_str = std::fs::read_to_string(&package_json_path).map_err(|e| {
                ApiError::InternalServerError(format!("Failed to read package.json: {e}"))
            })?;

            let package_json: Value = serde_json::from_str(&package_json_str).map_err(|e| {
                ApiError::InternalServerError(format!("Failed to parse package.json: {e}"))
            })?;

            Ok(Some(package_json))
        } else {
            Ok(None)
        }
    }

    fn generate_metadata_from_published_packages(
        package_name: &str,
        published_packages: &[Package],
        state: &AppState,
        request_host: Option<&str>,
        request_scheme: &str,
    ) -> Result<Value, ApiError> {
        use serde_json::json;
        use std::collections::HashMap;

        let mut versions = HashMap::new();
        let mut dist_tags = HashMap::new();
        let mut latest_version = "0.0.0".to_string();
        let mut package_description = None;

        // Get package with versions for each published package
        for pkg in published_packages {
            if let Some(pkg_with_versions) = state
                .database
                .get_package_with_versions(&pkg.name)
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?
            {
                // Process each version
                for version_with_files in pkg_with_versions.versions {
                    let version = version_with_files.version.version.clone();

                    // Load package.json from filesystem
                    if let Some(package_json) =
                        Self::load_package_json_from_filesystem(package_name, &version, state)?
                    {
                        // Update latest version (simple string comparison for now)
                        if version > latest_version {
                            latest_version = version.clone();
                        }

                        // Set description from first package if not set
                        if package_description.is_none() {
                            package_description = package_json
                                .get("description")
                                .and_then(|d| d.as_str())
                                .map(|s| s.to_string());
                        }

                        // Get the first file for the tarball URL
                        if let Some(file) = version_with_files.files.first() {
                            // Create version metadata
                            // Use request host if available, otherwise fall back to config host
                            let host_to_use = request_host.unwrap_or(&state.config.host);
                            let tarball_url = format!(
                                "{}://{}/registry/{}/-/{}",
                                request_scheme, host_to_use, package_name, file.filename
                            );

                            let mut version_data = package_json.clone();

                            // Ensure dist field exists with correct tarball URL
                            if let Some(dist) = version_data.get_mut("dist") {
                                if let Some(dist_obj) = dist.as_object_mut() {
                                    dist_obj.insert("tarball".to_string(), json!(tarball_url));
                                }
                            } else {
                                version_data["dist"] = json!({
                                    "tarball": tarball_url
                                });
                            }

                            versions.insert(version, version_data);
                        }
                    }
                }
            }
        }

        // Set dist-tags
        dist_tags.insert("latest".to_string(), latest_version);

        // Create the complete package metadata
        let metadata = json!({
            "name": package_name,
            "description": package_description.unwrap_or_else(|| "".to_string()),
            "dist-tags": dist_tags,
            "versions": versions,
            "_id": package_name,
            "_rev": "1-0"
        });

        Ok(metadata)
    }
}

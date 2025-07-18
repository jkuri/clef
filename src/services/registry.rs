use rocket::serde::json::Value;
use log::{info, error, debug, warn};
use crate::error::ApiError;
use crate::state::AppState;
use crate::config::AppConfig;
use crate::models::Package;
use diesel::prelude::*;

pub struct RegistryService;

impl RegistryService {
    fn rewrite_tarball_urls(json: &mut Value, config: &AppConfig, scheme: &str) -> Result<(), ApiError> {
        // Rewrite tarball URLs in package metadata to point to our proxy server
        if let Some(versions) = json.get_mut("versions").and_then(|v| v.as_object_mut()) {
            for (version, version_data) in versions.iter_mut() {
                if let Some(dist) = version_data.get_mut("dist").and_then(|d| d.as_object_mut()) {
                    if let Some(tarball_url) = dist.get("tarball").and_then(|t| t.as_str()).map(|s| s.to_string()) {
                        // Extract package name and filename from the original tarball URL
                        // Expected format: https://registry.npmjs.org/package/-/package-version.tgz
                        if tarball_url.starts_with("https://registry.npmjs.org/") {
                            if let Some(path_part) = tarball_url.strip_prefix("https://registry.npmjs.org/") {
                                // Rewrite to our proxy server URL using the same scheme as the request
                                let new_url = format!("{}://{}:{}/{}",
                                    scheme, config.host, config.port, path_part);

                                dist.insert("tarball".to_string(), Value::String(new_url.clone()));
                                debug!("Rewrote tarball URL for {}: {} -> {}",
                                    version, tarball_url, new_url);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn get_package_metadata(package: &str, state: &AppState) -> Result<Value, ApiError> {
        info!("Fetching metadata for package: {}", package);

        // Check metadata cache first
        if let Some(cache_entry) = state.cache.get_metadata(package).await {
            info!("Metadata cache hit for package: {} (size: {} bytes)", package, cache_entry.data.len());
            let metadata_str = String::from_utf8(cache_entry.data)
                .map_err(|e| ApiError::InternalServerError(format!("Invalid UTF-8 in cached metadata: {}", e)))?;
            let metadata: Value = serde_json::from_str(&metadata_str)
                .map_err(|e| ApiError::InternalServerError(format!("Invalid JSON in cached metadata: {}", e)))?;
            return Ok(metadata);
        }

        info!("Metadata cache miss for package: {}, generating fresh metadata", package);

        // First check if we have any published versions of this package in our database
        let mut conn = state.database.get_connection()
            .map_err(|e| ApiError::InternalServerError(format!("Database connection error: {}", e)))?;

        use crate::schema::packages;
        let published_packages: Vec<Package> = packages::table
            .filter(packages::name.eq(package))
            .filter(packages::author_id.is_not_null()) // Only published packages have author_id
            .load::<Package>(&mut conn)
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {}", e)))?;

        let metadata = if !published_packages.is_empty() {
            // We have published versions, generate metadata from our database
            info!("Found {} published versions for package: {}", published_packages.len(), package);
            Self::generate_metadata_from_published_packages(package, &published_packages, state)?
        } else {
            // No published versions found, proxy to upstream
            let url = format!("{}/{}", state.config.upstream_registry, package);

            // Check if we have cached metadata with ETag for conditional request
            let mut request = state.client.get(&url);

            // Add If-None-Match header if we have cached ETag
            if let Some(cache_entry) = state.cache.get_metadata(package).await {
                if let Some(etag) = &cache_entry.etag {
                    debug!("Adding If-None-Match header for upstream request: {}", etag);
                    request = request.header("If-None-Match", etag);
                }
            }

            let response = request.send().await?;

            if response.status() == 304 {
                // Not Modified - use cached version
                debug!("Upstream returned 304 Not Modified for package: {}", package);
                if let Some(cache_entry) = state.cache.get_metadata(package).await {
                    info!("Using cached metadata after 304 Not Modified for package: {} (size: {} bytes)", package, cache_entry.data.len());
                    let metadata_str = String::from_utf8(cache_entry.data)
                        .map_err(|e| ApiError::InternalServerError(format!("Invalid UTF-8 in cached metadata: {}", e)))?;
                    let metadata: Value = serde_json::from_str(&metadata_str)
                        .map_err(|e| ApiError::InternalServerError(format!("Invalid JSON in cached metadata: {}", e)))?;
                    return Ok(metadata);
                } else {
                    return Err(ApiError::InternalServerError("Received 304 but no cached metadata found".to_string()));
                }
            } else if response.status().is_success() {
                // Extract ETag for future conditional requests
                let etag = response.headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                match response.json::<Value>().await {
                    Ok(mut json) => {
                        // Rewrite tarball URLs to point to our proxy server
                        let scheme = state.config.get_scheme();
                        Self::rewrite_tarball_urls(&mut json, &state.config, scheme)?;

                        info!("Successfully proxied metadata for package: {}", package);

                        // Cache with ETag if available
                        let metadata_str = serde_json::to_string(&json)
                            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize metadata for caching: {}", e)))?;

                        if let Err(e) = state.cache.put_metadata_with_etag(package, &metadata_str, etag.as_deref()).await {
                            warn!("Failed to cache metadata for package {}: {}", package, e);
                        }

                        return Ok(json);
                    }
                    Err(e) => {
                        error!("Failed to parse JSON response for package {}: {}", package, e);
                        return Err(ApiError::ParseError(format!("Failed to parse upstream response: {}", e)));
                    }
                }
            } else {
                error!("Upstream returned error {} for package: {}", response.status(), package);
                return Err(ApiError::UpstreamError(format!("Upstream error: {}", response.status())));
            }
        };

        // Cache the metadata
        let metadata_str = serde_json::to_string(&metadata)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize metadata for caching: {}", e)))?;

        if let Err(e) = state.cache.put_metadata(package, &metadata_str).await {
            warn!("Failed to cache metadata for package {}: {}", package, e);
        }

        Ok(metadata)
    }

    pub async fn get_package_version_metadata(package: &str, version: &str, state: &AppState) -> Result<Value, ApiError> {
        info!("Fetching metadata for package: {} version: {}", package, version);

        let url = format!("{}/{}/{}", state.config.upstream_registry, package, version);

        let response = state.client.get(&url).send().await?;

        if response.status().is_success() {
            match response.json::<Value>().await {
                Ok(json) => {
                    info!("Successfully proxied metadata for package: {} version: {}", package, version);
                    Ok(json)
                }
                Err(e) => {
                    error!("Failed to parse JSON response for package {} version {}: {}", package, version, e);
                    Err(ApiError::ParseError(format!("Failed to parse upstream response: {}", e)))
                }
            }
        } else {
            error!("Upstream returned error {} for package: {} version: {}", response.status(), package, version);
            Err(ApiError::UpstreamError(format!("Upstream error: {}", response.status())))
        }
    }

    pub async fn get_package_tarball(package: &str, filename: &str, state: &AppState) -> Result<Vec<u8>, ApiError> {
        info!("Fetching tarball for package: {} filename: {}", package, filename);

        // Check cache first
        if let Some(cache_entry) = state.cache.get(package, filename).await {
            info!("Cache hit for tarball: {} filename: {} (size: {} bytes)", package, filename, cache_entry.data.len());
            return Ok(cache_entry.data);
        }

        // Cache miss, fetch from upstream
        let url = format!("{}/{}/-/{}", state.config.upstream_registry, package, filename);

        let response = state.client.get(&url).send().await?;

        if response.status().is_success() {
            // Extract ETag for cache validation
            let etag = response.headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match response.bytes().await {
                Ok(bytes) => {
                    let data = bytes.to_vec();

                    // Store in cache
                    if let Err(e) = state.cache.put(package, filename, &data, etag.as_deref(), &url, Some(&*state.database)).await {
                        error!("Failed to cache tarball for {} filename {}: {}", package, filename, e);
                    }

                    info!("Successfully proxied and cached tarball for package: {} filename: {} (size: {} bytes)",
                          package, filename, data.len());
                    Ok(data)
                }
                Err(e) => {
                    error!("Failed to read bytes from response for package {} filename {}: {}", package, filename, e);
                    Err(ApiError::ParseError(format!("Failed to read upstream response: {}", e)))
                }
            }
        } else {
            error!("Upstream returned error {} for package: {} filename: {}", response.status(), package, filename);
            Err(ApiError::UpstreamError(format!("Upstream error: {}", response.status())))
        }
    }

    pub async fn head_package_tarball(package: &str, filename: &str, state: &AppState) -> Result<(), ApiError> {
        info!("HEAD request for tarball: {} filename: {}", package, filename);

        // Check cache first
        if state.cache.get(package, filename).await.is_some() {
            info!("Cache hit for HEAD tarball: {} filename: {}", package, filename);
            return Ok(());
        }

        // Cache miss, check upstream
        let url = format!("{}/{}/-/{}", state.config.upstream_registry, package, filename);

        let response = state.client.head(&url).send().await?;

        if response.status().is_success() {
            info!("Successfully checked tarball for package: {} filename: {}", package, filename);
            Ok(())
        } else {
            error!("Upstream returned error {} for HEAD package: {} filename: {}", response.status(), package, filename);
            Err(ApiError::UpstreamError(format!("Upstream error: {}", response.status())))
        }
    }

    fn generate_metadata_from_published_packages(
        package_name: &str,
        published_packages: &[Package],
        state: &AppState,
    ) -> Result<Value, ApiError> {
        use std::collections::HashMap;
        use serde_json::json;

        let mut versions = HashMap::new();
        let mut dist_tags = HashMap::new();
        let mut latest_version = "0.0.0".to_string();
        let mut package_description = None;

        // Process each published version
        for pkg in published_packages {
            if let Some(package_json_str) = &pkg.package_json {
                // Parse the stored package.json
                let package_json: Value = serde_json::from_str(package_json_str)
                    .map_err(|e| ApiError::InternalServerError(format!("Failed to parse package.json: {}", e)))?;

                // Extract version info
                let version = pkg.version.clone();

                // Update latest version (simple string comparison for now)
                if version > latest_version {
                    latest_version = version.clone();
                }

                // Set description from first package if not set
                if package_description.is_none() {
                    package_description = package_json.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                }

                // Create version metadata
                let scheme = state.config.get_scheme();
                let tarball_url = format!("{}://{}:{}/{}/-/{}",
                    scheme, state.config.host, state.config.port, package_name, pkg.filename);

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

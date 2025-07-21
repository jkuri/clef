use crate::error::ApiError;
use crate::models::{AuthenticatedUser, NpmPublishRequest, NpmPublishResponse};
use crate::state::AppState;
use log::{debug, warn};
use rocket::serde::json::Json;
use rocket::{State, put};

/// npm publish endpoint - PUT /registry/:package
#[put("/registry/<package>", data = "<publish_request>")]
pub async fn npm_publish(
    package: &str,
    publish_request: Json<NpmPublishRequest>,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<NpmPublishResponse>, ApiError> {
    use base64::prelude::*;
    use std::fs;
    use std::path::Path;

    debug!(
        "Publishing package: {} (URL parameter: {})",
        publish_request.name, package
    );
    debug!(
        "Request has {} versions and {} attachments",
        publish_request.versions.len(),
        publish_request._attachments.len()
    );

    // Validate that the package name in the URL matches the one in the request
    if publish_request.name != package {
        return Err(ApiError::BadRequest(format!(
            "Package name mismatch: URL has '{}' but request has '{}'",
            package, publish_request.name
        )));
    }

    // Validate that we have at least one version and one attachment
    if publish_request.versions.is_empty() {
        return Err(ApiError::BadRequest(
            "No versions provided in publish request".to_string(),
        ));
    }

    if publish_request._attachments.is_empty() {
        return Err(ApiError::BadRequest(
            "No attachments provided in publish request".to_string(),
        ));
    }

    // Check if user has permission to publish this package
    // Check if user can publish to this package
    let can_publish = state
        .database
        .can_publish_package(package, user.user_id)
        .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

    if !can_publish {
        return Err(ApiError::Forbidden(format!(
            "User {} does not have permission to publish package '{}'",
            user.user_id, package
        )));
    }

    // Check if this is a new package (no existing owners)
    let is_new_package = !state
        .database
        .package_exists(package)
        .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

    // Get the first version from the request (npm publish sends one version at a time)
    let (version, version_data) = publish_request
        .versions
        .iter()
        .next()
        .ok_or_else(|| ApiError::BadRequest("No version data provided".to_string()))?;

    debug!("Publishing version: {version}");

    // Create or get the package in the database
    let pkg = state
        .database
        .create_or_get_package_with_update(
            package,
            version_data.description.clone(),
            Some(user.user_id),
            true, // Update description if provided
        )
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    // Update package metadata (license, etc.) from version data
    if version_data.license.is_some() {
        state
            .database
            .update_package_metadata(
                pkg.id,
                None, // homepage
                None, // repository_url
                version_data.license.clone(),
                None, // keywords
            )
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to update package metadata: {e}"))
            })?;
    }

    // Update package privacy if specified in the publish request
    if let Some(is_private) = publish_request.private {
        state
            .database
            .update_package_privacy(package, is_private)
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to update package privacy: {e}"))
            })?;
    }

    debug!("Package ID: {}", pkg.id);

    // Create or get the package version
    let version_json = serde_json::to_value(version_data).map_err(|e| {
        ApiError::InternalServerError(format!("Failed to serialize version data: {e}"))
    })?;

    let pkg_version = state
        .database
        .create_or_get_package_version_with_metadata(pkg.id, version, &version_json)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {e}")))?;

    debug!("Package version ID: {}", pkg_version.id);

    // Process attachments (tarballs)
    for (filename, attachment) in &publish_request._attachments {
        debug!("Processing attachment: {filename}");

        // Decode the base64 data
        let tarball_data = BASE64_STANDARD
            .decode(&attachment.data)
            .map_err(|e| ApiError::BadRequest(format!("Invalid base64 data: {e}")))?;

        debug!("Decoded tarball size: {} bytes", tarball_data.len());

        // Create packages directory structure
        // Scoped packages like @jkuri/test-scoped-package are stored as @jkuri/test-scoped-package/
        let cache_dir = Path::new(&state.config.cache_dir);
        let packages_dir = cache_dir.join("packages");
        let package_dir = packages_dir.join(package);

        debug!("Package name: {package}");
        debug!("Package directory: {package_dir:?}");
        debug!("Creating directory: {package_dir:?}");
        fs::create_dir_all(&package_dir).map_err(|e| {
            debug!("Failed to create directory {package_dir:?}: {e}");
            ApiError::InternalServerError(format!("Failed to create package directory: {e}"))
        })?;

        // Save the tarball
        // For scoped packages like @jkuri/test-scoped-package, the tarball filename should be test-scoped-package-1.0.0.tgz
        let tarball_filename = if package.starts_with('@') {
            // Extract the package name without the scope for the filename
            let package_name = package.split('/').next_back().unwrap_or(package);
            format!("{package_name}-{version}.tgz")
        } else {
            format!("{package}-{version}.tgz")
        };
        let tarball_path = package_dir.join(&tarball_filename);
        debug!("Writing tarball to: {tarball_path:?}");
        fs::write(&tarball_path, &tarball_data).map_err(|e| {
            debug!("Failed to write tarball to {tarball_path:?}: {e}");
            ApiError::InternalServerError(format!("Failed to write tarball: {e}"))
        })?;

        // Store package.json to filesystem instead of database
        let package_json = serde_json::to_string(&version_data).map_err(|e| {
            ApiError::InternalServerError(format!("Failed to serialize package.json: {e}"))
        })?;

        // Save package.json alongside the tarball
        let package_json_path = package_dir.join(format!(
            "{}-{}.json",
            if package.starts_with('@') {
                package.split('/').next_back().unwrap_or(package)
            } else {
                package
            },
            version
        ));
        fs::write(&package_json_path, &package_json).map_err(|e| {
            ApiError::InternalServerError(format!("Failed to write package.json: {e}"))
        })?;

        debug!("Wrote tarball to: {}", tarball_path.display());

        // Store file information in database
        let upstream_url = format!(
            "{}/{}/-/{}",
            state.config.upstream_registry, package, tarball_filename
        );

        state
            .database
            .create_or_update_package_file(
                pkg_version.id,
                &tarball_filename,
                tarball_data.len() as i64,
                &upstream_url,
                &tarball_path.to_string_lossy(),
                None,                                         // etag
                Some("application/octet-stream".to_string()), // content_type
            )
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to create package file: {e}"))
            })?;
    }

    // If this is a new package, create ownership record
    if is_new_package {
        state
            .database
            .create_package_owner(package, user.user_id, "admin")
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to create ownership: {e}"))
            })?;
    }

    // Invalidate metadata cache since we've published a new version
    if let Err(e) = state.cache.invalidate_metadata(package).await {
        warn!("Failed to invalidate metadata cache for package {package}: {e}");
    }

    Ok(Json(NpmPublishResponse {
        ok: true,
        id: package.to_string(),
        rev: "1-0".to_string(),
    }))
}

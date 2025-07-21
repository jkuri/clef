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

        // Create cache directory if it doesn't exist
        let cache_dir = Path::new(&state.config.cache_dir);
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir).map_err(|e| {
                ApiError::InternalServerError(format!("Failed to create cache directory: {e}"))
            })?;
        }

        // Create package-specific directory
        let package_dir = cache_dir.join(package);
        if !package_dir.exists() {
            fs::create_dir_all(&package_dir).map_err(|e| {
                ApiError::InternalServerError(format!("Failed to create package directory: {e}"))
            })?;
        }

        // Write the tarball to the cache
        let tarball_path = package_dir.join(filename);
        fs::write(&tarball_path, &tarball_data)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to write tarball: {e}")))?;

        debug!("Wrote tarball to: {}", tarball_path.display());

        // Store file information in database
        let upstream_url = format!(
            "{}/{}/-/{}",
            state.config.upstream_registry, package, filename
        );

        state
            .database
            .create_or_update_package_file(
                pkg_version.id,
                filename,
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

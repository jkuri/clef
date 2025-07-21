use crate::error::ApiError;
use crate::models::{
    AuthenticatedUser, LoginRequest, LogoutResponse, NewPackageOwner, NpmPublishRequest,
    NpmPublishResponse, NpmUserDocument, NpmUserResponse, RegisterRequest, WhoamiResponse,
};
use crate::services::AuthService;
use crate::state::AppState;
use log::{debug, warn};
use rocket::serde::Serialize;
use rocket::{State, put, serde::json::Json};

#[derive(Serialize, Debug)]
pub struct NpmErrorResponse {
    pub error: String,
    pub reason: String,
}

// npm login endpoint - PUT /registry/-/user/org.couchdb.user:username
#[put("/registry/-/user/<user_id>", data = "<user_doc>")]
pub async fn npm_login(
    user_id: &str,
    user_doc: Json<NpmUserDocument>,
    state: &State<AppState>,
) -> Result<Json<NpmUserResponse>, ApiError> {
    // Validate the user_id format (should be org.couchdb.user:username)
    if !user_id.starts_with("org.couchdb.user:") {
        return Err(ApiError::BadRequest("Invalid user ID format".to_string()));
    }

    let username = user_id
        .strip_prefix("org.couchdb.user:")
        .ok_or_else(|| ApiError::BadRequest("Invalid user ID format".to_string()))?;

    // Validate that the username matches the document
    if user_doc.name != username {
        return Err(ApiError::BadRequest("Username mismatch".to_string()));
    }

    // Check if this is a login (existing user) or registration (new user)
    let existing_user = AuthService::get_user_by_username(&state.database, username)?;

    if let Some(_user) = existing_user {
        // Existing user - authenticate
        let login_request = LoginRequest {
            name: user_doc.name.clone(),
            password: user_doc.password.clone(),
        };

        let (_user, token) = AuthService::authenticate_user(&state.database, login_request)?;

        Ok(Json(NpmUserResponse {
            ok: true,
            id: user_id.to_string(),
            rev: "1-0".to_string(), // npm expects a revision
            token,
        }))
    } else {
        // New user - register
        let email = user_doc
            .email
            .clone()
            .unwrap_or_else(|| format!("{username}@example.com"));

        let register_request = RegisterRequest {
            name: user_doc.name.clone(),
            email,
            password: user_doc.password.clone(),
        };

        let _user = AuthService::register_user(&state.database, register_request)?;

        // Create authentication token for the new user
        let login_request = LoginRequest {
            name: user_doc.name.clone(),
            password: user_doc.password.clone(),
        };

        let (_user, token) = AuthService::authenticate_user(&state.database, login_request)?;

        Ok(Json(NpmUserResponse {
            ok: true,
            id: user_id.to_string(),
            rev: "1-0".to_string(),
            token,
        }))
    }
}

use rocket::{delete, get};

#[get("/registry/-/whoami")]
pub async fn npm_whoami(user: AuthenticatedUser) -> Json<WhoamiResponse> {
    Json(WhoamiResponse {
        username: user.username,
    })
}

// npm logout endpoint - DELETE /registry/-/user/token/{token}
#[delete("/registry/-/user/token/<token>")]
pub async fn npm_logout(
    token: &str,
    state: &State<AppState>,
) -> Result<Json<LogoutResponse>, ApiError> {
    // Revoke the token
    AuthService::revoke_token(&state.database, token)?;

    Ok(Json(LogoutResponse { ok: true }))
}

// Simple login and register endpoints have been moved to src/routes/api.rs with /api/v1/ prefix
// This file now only contains npm-specific authentication routes

// npm publish endpoint - PUT /registry/:package
#[put("/registry/<package>", data = "<publish_request>")]
pub async fn npm_publish(
    package: &str,
    publish_request: Json<NpmPublishRequest>,
    user: AuthenticatedUser,
    state: &State<AppState>,
) -> Result<Json<NpmPublishResponse>, ApiError> {
    use crate::schema::package_owners;
    use base64::prelude::*;
    use diesel::prelude::*;
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

    // Validate package name matches request
    if publish_request.name != package {
        return Err(ApiError::BadRequest(format!(
            "Package name mismatch: expected '{}', got '{}'",
            publish_request.name, package
        )));
    }

    // Check if user has permission to publish this package
    let mut conn = state
        .database
        .get_connection()
        .map_err(|e| ApiError::InternalServerError(format!("Database connection error: {e}")))?;

    // For new packages, user automatically gets ownership
    // For existing packages, check ownership
    let existing_owner = package_owners::table
        .filter(package_owners::package_name.eq(package))
        .filter(package_owners::user_id.eq(user.user_id))
        .first::<crate::models::PackageOwner>(&mut conn)
        .optional()
        .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

    let is_new_package = existing_owner.is_none();

    // Get the first version from the request (npm publish sends one version at a time)
    let (version, version_data) = publish_request
        .versions
        .iter()
        .next()
        .ok_or_else(|| ApiError::BadRequest("No version data provided".to_string()))?;

    debug!("Publishing version: {version}");

    // Get the attachment (tarball)
    // npm sends the attachment key using the full package name, including scope
    let attachment_key = format!("{package}-{version}.tgz");
    debug!("Looking for attachment with key: {attachment_key}");
    debug!(
        "Available attachment keys: {:?}",
        publish_request._attachments.keys().collect::<Vec<_>>()
    );

    let attachment = publish_request
        ._attachments
        .get(&attachment_key)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "No tarball attachment found with key '{}'. Available keys: {:?}",
                attachment_key,
                publish_request._attachments.keys().collect::<Vec<_>>()
            ))
        })?;

    // Decode the base64 tarball data
    let tarball_data = BASE64_STANDARD
        .decode(&attachment.data)
        .map_err(|e| ApiError::BadRequest(format!("Invalid base64 data: {e}")))?;

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
    fs::write(&package_json_path, &package_json)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to write package.json: {e}")))?;

    // Create or get the package first, forcing description update for existing packages
    let pkg = state
        .database
        .create_or_get_package_with_update(
            package,
            version_data.description.clone(),
            Some(user.user_id),
            true, // force update description when publishing
        )
        .map_err(|e| ApiError::InternalServerError(format!("Failed to create package: {e}")))?;

    // Create the package version with full metadata from package.json
    let version_json = serde_json::to_value(version_data).map_err(|e| {
        ApiError::InternalServerError(format!("Failed to serialize version data: {e}"))
    })?;

    // Force update metadata when publishing new versions over existing ones
    let package_version = state
        .database
        .create_or_get_package_version_with_metadata_and_update(
            pkg.id,
            version,
            &version_json,
            true,
        )
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to create package version: {e}"))
        })?;

    // Create the package file entry
    let _package_file = state
        .database
        .create_or_update_package_file(
            package_version.id,
            &tarball_filename,
            tarball_data.len() as i64,
            format!("published://{package}/{version}").as_str(),
            tarball_path.to_string_lossy().as_ref(),
            None,                                         // etag
            Some("application/octet-stream".to_string()), // content_type
        )
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to create package file: {e}"))
        })?;

    // If this is a new package, create ownership record
    if is_new_package {
        let new_owner =
            NewPackageOwner::new(package.to_string(), user.user_id, "admin".to_string());

        diesel::insert_into(package_owners::table)
            .values(&new_owner)
            .execute(&mut conn)
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

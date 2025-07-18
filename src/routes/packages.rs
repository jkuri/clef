use rocket::serde::json::Value;
use rocket::{get, head, State, request::{Request, FromRequest, Outcome, FromParam}, response::Responder, Response};
use rocket::http::{Status, ContentType};
use std::io::Cursor;
use log;
use crate::state::AppState;
use crate::error::ApiError;
use crate::services::RegistryService;

// Custom request guard to extract URI path
pub struct UriPath(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UriPath {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let path = request.uri().path().as_str().to_string();
        Outcome::Success(UriPath(path))
    }
}

// Custom responder that can handle both JSON and binary responses
#[derive(Debug)]
pub enum PackageResponse {
    Json(Value),
    Binary(Vec<u8>),
    Empty,
}

impl<'r> Responder<'r, 'static> for PackageResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            PackageResponse::Json(json) => {
                Response::build()
                    .header(ContentType::JSON)
                    .sized_body(json.to_string().len(), Cursor::new(json.to_string()))
                    .ok()
            },
            PackageResponse::Binary(data) => {
                Response::build()
                    .header(ContentType::Binary)
                    .sized_body(data.len(), Cursor::new(data))
                    .ok()
            },
            PackageResponse::Empty => {
                Response::build()
                    .status(Status::Ok)
                    .ok()
            }
        }
    }
}

// Helper function to decode URL-encoded package names
fn decode_package_name(encoded: &str) -> String {
    // Handle URL-encoded scoped packages: %40types%2Fnode -> @types/node
    // Also handle other common URL encodings
    encoded
        .replace("%40", "@")
        .replace("%2F", "/")
        .replace("%2f", "/") // lowercase variant
        .replace("%20", " ")
        .replace("%2B", "+")
        .replace("%2b", "+") // lowercase variant
}

// Parse package request from URI path
fn parse_package_path(path: &str) -> Option<(String, PackageRequestType)> {
    // First decode the entire path to handle URL-encoded characters
    let decoded_path = decode_package_name(path);
    let segments: Vec<&str> = decoded_path.trim_start_matches('/').split('/').collect();

    if segments.is_empty() {
        return None;
    }

    // Handle scoped packages: @scope/name/...
    if segments[0].starts_with('@') && segments.len() >= 2 {
        let package_name = format!("{}/{}", segments[0], segments[1]);
        let remaining = &segments[2..];

        match remaining {
            [] => Some((package_name, PackageRequestType::Metadata)),
            [version] if !version.starts_with('-') => Some((package_name, PackageRequestType::Version(version.to_string()))),
            ["-", filename] => Some((package_name, PackageRequestType::Tarball(filename.to_string()))),
            _ => None,
        }
    } else {
        // Handle regular packages: name/...
        let package_name = segments[0].to_string();
        let remaining = &segments[1..];

        match remaining {
            [] => Some((package_name, PackageRequestType::Metadata)),
            [version] if !version.starts_with('-') => Some((package_name, PackageRequestType::Version(version.to_string()))),
            ["-", filename] => Some((package_name, PackageRequestType::Tarball(filename.to_string()))),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum PackageRequestType {
    Metadata,
    Version(String),
    Tarball(String),
}





// Specific routes for scoped packages (higher priority)
// Route for scoped package metadata: @scope/package
#[get("/<scope>/<package>", rank = 1)]
pub async fn handle_scoped_package_metadata(scope: ScopedPackageName, package: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    let full_package_name = format!("{}/{}", scope.0, package);
    log::info!("Scoped package metadata request: {}", full_package_name);
    let result = RegistryService::get_package_metadata(&full_package_name, state).await?;
    Ok(PackageResponse::Json(result))
}

// Custom parameter type that only matches scoped package names (starting with @)
pub struct ScopedPackageName(pub String);

impl<'r> FromParam<'r> for ScopedPackageName {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        if param.starts_with('@') {
            Ok(ScopedPackageName(param.to_string()))
        } else {
            Err(param)
        }
    }
}

// Route for scoped package version: @scope/package/version
// Only match when scope actually starts with @
#[get("/<scope>/<package>/<version>", rank = 1)]
pub async fn handle_scoped_package_version(scope: ScopedPackageName, package: &str, version: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    let full_package_name = format!("{}/{}", scope.0, package);
    log::info!("Scoped package version request: {} version {}", full_package_name, version);
    let result = RegistryService::get_package_version_metadata(&full_package_name, version, state).await?;
    Ok(PackageResponse::Json(result))
}

// Route for scoped package tarball: @scope/package/-/filename
#[get("/<scope>/<package>/-/<filename>", rank = 1)]
pub async fn handle_scoped_package_tarball(scope: ScopedPackageName, package: &str, filename: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    let full_package_name = format!("{}/{}", scope.0, package);
    log::info!("Scoped package tarball request: {} file {}", full_package_name, filename);
    let result = RegistryService::get_package_tarball(&full_package_name, filename, state).await?;
    Ok(PackageResponse::Binary(result))
}

// HEAD request for scoped package tarballs
#[head("/<scope>/<package>/-/<filename>", rank = 1)]
pub async fn handle_scoped_package_tarball_head(scope: ScopedPackageName, package: &str, filename: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    let full_package_name = format!("{}/{}", scope.0, package);
    log::info!("Scoped package tarball HEAD request: {} file {}", full_package_name, filename);
    RegistryService::head_package_tarball(&full_package_name, filename, state).await?;
    Ok(PackageResponse::Empty)
}

// Regular package routes (lower priority)
// Route for regular package metadata: package
#[get("/<package>", rank = 2)]
pub async fn handle_regular_package_metadata(package: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    log::info!("Regular package metadata handler received: '{}'", package);

    // Check if this is a decoded scoped package (starts with @ and contains /)
    // This happens when npm sends @types%2fnode-forge and Rocket decodes it to @types/node-forge
    if package.starts_with('@') && package.contains('/') {
        log::info!("Decoded scoped package metadata request: {}", package);
        let result = RegistryService::get_package_metadata(package, state).await?;
        return Ok(PackageResponse::Json(result));
    }
    // Skip if this looks like a regular scoped package (starts with @ but no /)
    if package.starts_with('@') {
        log::info!("Rejecting malformed scoped package: {}", package);
        return Err(ApiError::BadRequest("Invalid scoped package format".to_string()));
    }
    log::info!("Regular package metadata request: {}", package);
    let result = RegistryService::get_package_metadata(package, state).await?;
    Ok(PackageResponse::Json(result))
}

// Route for regular package version: package/version
#[get("/<package>/<version>", rank = 2)]
pub async fn handle_regular_package_version(package: &str, version: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    // Skip if this looks like a scoped package (starts with @)
    if package.starts_with('@') {
        return Err(ApiError::BadRequest("Use scoped package route".to_string()));
    }
    log::info!("Regular package version request: {} version {}", package, version);
    let result = RegistryService::get_package_version_metadata(package, version, state).await?;
    Ok(PackageResponse::Json(result))
}

// Route for regular package tarball: package/-/filename
#[get("/<package>/-/<filename>", rank = 2)]
pub async fn handle_regular_package_tarball(package: &str, filename: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    // Skip if this looks like a scoped package (starts with @)
    if package.starts_with('@') {
        return Err(ApiError::BadRequest("Use scoped package route".to_string()));
    }
    log::info!("Regular package tarball request: {} file {}", package, filename);
    let result = RegistryService::get_package_tarball(package, filename, state).await?;
    Ok(PackageResponse::Binary(result))
}

// HEAD request for regular package tarballs
#[head("/<package>/-/<filename>", rank = 2)]
pub async fn handle_regular_package_tarball_head(package: &str, filename: &str, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    // Skip if this looks like a scoped package (starts with @)
    if package.starts_with('@') {
        return Err(ApiError::BadRequest("Use scoped package route".to_string()));
    }
    log::info!("Regular package tarball HEAD request: {} file {}", package, filename);
    RegistryService::head_package_tarball(package, filename, state).await?;
    Ok(PackageResponse::Empty)
}

// Catch-all route for any remaining requests (lowest priority)
#[get("/<path..>", rank = 3)]
pub async fn handle_package_request(path: std::path::PathBuf, uri_path: UriPath, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    log::info!("Package request received: {} (path: {})", uri_path.0, path.display());

    if let Some((package_name, request_type)) = parse_package_path(&uri_path.0) {
        log::info!("Parsed package: {} with request type: {:?}", package_name, request_type);
        match request_type {
            PackageRequestType::Metadata => {
                let result = RegistryService::get_package_metadata(&package_name, state).await?;
                Ok(PackageResponse::Json(result))
            },
            PackageRequestType::Version(version) => {
                let result = RegistryService::get_package_version_metadata(&package_name, &version, state).await?;
                Ok(PackageResponse::Json(result))
            },
            PackageRequestType::Tarball(filename) => {
                let result = RegistryService::get_package_tarball(&package_name, &filename, state).await?;
                Ok(PackageResponse::Binary(result))
            }
        }
    } else {
        log::warn!("Failed to parse package path: {}", uri_path.0);
        Err(ApiError::BadRequest("Invalid package path".to_string()))
    }
}

// HEAD request handler
#[head("/<_path..>")]
pub async fn handle_package_head_request(_path: std::path::PathBuf, uri_path: UriPath, state: &State<AppState>) -> Result<PackageResponse, ApiError> {
    if let Some((package_name, request_type)) = parse_package_path(&uri_path.0) {
        match request_type {
            PackageRequestType::Tarball(filename) => {
                RegistryService::head_package_tarball(&package_name, &filename, state).await?;
                Ok(PackageResponse::Empty)
            },
            _ => Err(ApiError::BadRequest("HEAD only supported for tarballs".to_string()))
        }
    } else {
        Err(ApiError::BadRequest("Invalid package path".to_string()))
    }
}

use crate::error::ApiError;
use crate::models::{CacheAnalytics, PackageListResponse, PackageVersionsResponse, PopularPackage};
use crate::state::AppState;
use rocket::serde::json::Json;
use rocket::{State, get};

#[get("/packages")]
pub async fn list_packages(state: &State<AppState>) -> Result<Json<PackageListResponse>, ApiError> {
    let packages = state
        .database
        .get_all_packages_with_versions()
        .map_err(|e| ApiError::ParseError(format!("Failed to list packages: {e}")))?;

    let total_count = packages.len();

    // Calculate total size from all files across all versions
    let total_size_bytes = packages
        .iter()
        .flat_map(|pkg| &pkg.versions)
        .flat_map(|ver| &ver.files)
        .map(|file| file.size_bytes)
        .sum::<i64>();

    let total_size_mb = total_size_bytes as f64 / 1024.0 / 1024.0;

    Ok(Json(PackageListResponse {
        packages,
        total_count,
        total_size_bytes,
        total_size_mb,
    }))
}

#[get("/packages/<name>")]
pub async fn get_package_versions(
    name: &str,
    state: &State<AppState>,
) -> Result<Json<PackageVersionsResponse>, ApiError> {
    let package_with_versions = state
        .database
        .get_package_with_versions(name)
        .map_err(|e| ApiError::ParseError(format!("Failed to get package versions: {e}")))?;

    match package_with_versions {
        Some(pkg_with_versions) => {
            let total_size_bytes = pkg_with_versions
                .versions
                .iter()
                .flat_map(|ver| &ver.files)
                .map(|file| file.size_bytes)
                .sum::<i64>();

            Ok(Json(PackageVersionsResponse {
                package: pkg_with_versions.package,
                versions: pkg_with_versions.versions,
                total_size_bytes,
            }))
        }
        None => Err(ApiError::NotFound(format!("Package '{name}' not found"))),
    }
}

#[get("/packages/popular?<limit>")]
pub async fn get_popular_packages(
    limit: Option<i64>,
    state: &State<AppState>,
) -> Result<Json<Vec<PopularPackage>>, ApiError> {
    let limit = limit.unwrap_or(10);
    let popular_packages = state
        .database
        .get_popular_packages(limit)
        .map_err(|e| ApiError::ParseError(format!("Failed to get popular packages: {e}")))?;

    Ok(Json(popular_packages))
}

#[get("/analytics")]
pub async fn get_cache_analytics(
    state: &State<AppState>,
) -> Result<Json<CacheAnalytics>, ApiError> {
    let (total_packages, total_size_bytes) = state
        .database
        .get_cache_stats()
        .map_err(|e| ApiError::ParseError(format!("Failed to get cache stats: {e}")))?;

    let popular_packages = state
        .database
        .get_popular_packages(5)
        .map_err(|e| ApiError::ParseError(format!("Failed to get popular packages: {e}")))?;

    let recent_packages = state
        .database
        .get_recent_packages(10)
        .map_err(|e| ApiError::ParseError(format!("Failed to get recent packages: {e}")))?;

    let cache_hit_rate = state.cache.get_hit_rate();

    let analytics = CacheAnalytics {
        total_packages: total_packages as i64,
        total_size_bytes,
        total_size_mb: total_size_bytes as f64 / 1024.0 / 1024.0,
        most_popular_packages: popular_packages,
        recent_packages,
        cache_hit_rate,
    };

    Ok(Json(analytics))
}

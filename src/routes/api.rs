use crate::error::ApiError;
use crate::models::{
    CacheAnalytics, CacheStatsResponse, PackageListResponse, PackageVersionsResponse,
    PopularPackage,
};
use crate::state::AppState;
use log::{debug, info};
use rocket::serde::json::Json;
use rocket::{State, delete, get, post};
use serde_json;

// Import auth types from models
use crate::models::{LoginRequest, LoginResponse, NpmUserResponse, RegisterRequest};
use crate::services::auth::AuthService;

// Health check endpoint
#[get("/api/v1/health")]
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

// Analytics endpoints
#[get("/api/v1/packages?<limit>&<page>&<search>&<sort>&<order>")]
pub async fn list_packages(
    limit: Option<i64>,
    page: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    state: &State<AppState>,
) -> Result<Json<PackageListResponse>, ApiError> {
    let limit = limit.unwrap_or(20).clamp(1, 100); // Default 20, max 100
    let page = page.unwrap_or(1).max(1); // Default page 1, minimum 1
    let offset = (page - 1) * limit;

    let search_query = search.as_deref();
    let sort_column = sort.as_deref();
    let sort_order = order.as_deref();

    // Validate sort parameters
    let valid_columns = ["name", "created_at", "updated_at", "id"];
    let valid_orders = ["asc", "desc"];

    let sort_column = match sort_column {
        Some(col) if valid_columns.contains(&col) => Some(col),
        Some(_) => Some("created_at"), // Default to created_at for invalid columns
        None => None,
    };

    let sort_order = match sort_order {
        Some(ord) if valid_orders.contains(&ord) => Some(ord),
        Some(_) => Some("desc"), // Default to desc for invalid orders
        None => None,
    };

    let (packages, total_count) = state
        .database
        .get_packages_paginated(limit, offset, search_query, sort_column, sort_order)
        .map_err(|e| ApiError::ParseError(format!("Failed to list packages: {e}")))?;

    // Calculate total size from all files across all versions
    let total_size_bytes = packages
        .iter()
        .flat_map(|pkg| &pkg.versions)
        .flat_map(|ver| &ver.files)
        .map(|file| file.size_bytes)
        .sum::<i64>();

    let total_size_mb = total_size_bytes as f64 / 1024.0 / 1024.0;

    // Calculate pagination metadata
    let total_pages = (total_count + limit - 1) / limit; // Ceiling division
    let has_next = page < total_pages;
    let has_prev = page > 1;

    let pagination = crate::models::package::PaginationMetadata {
        page,
        limit,
        total_pages,
        has_next,
        has_prev,
    };

    Ok(Json(PackageListResponse {
        packages,
        total_count,
        total_size_bytes,
        total_size_mb,
        pagination,
    }))
}

#[get("/api/v1/packages/<name>")]
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

#[get("/api/v1/packages/popular?<limit>")]
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

#[get("/api/v1/analytics")]
pub async fn get_cache_analytics(
    state: &State<AppState>,
) -> Result<Json<CacheAnalytics>, ApiError> {
    info!("Analytics endpoint called");

    let (total_packages, db_size_bytes) = state
        .database
        .get_cache_stats()
        .map_err(|e| ApiError::ParseError(format!("Failed to get cache stats: {e}")))?;

    debug!("Database reports {total_packages} total packages, {db_size_bytes} bytes total size");

    let popular_packages = state
        .database
        .get_popular_packages(5)
        .map_err(|e| ApiError::ParseError(format!("Failed to get popular packages: {e}")))?;

    debug!("Retrieved {} popular packages", popular_packages.len());

    let recent_packages = state
        .database
        .get_recent_packages(10)
        .map_err(|e| ApiError::ParseError(format!("Failed to get recent packages: {e}")))?;

    debug!("Retrieved {} recent packages", recent_packages.len());

    let cache_hit_rate = state.cache.get_hit_rate();
    debug!("Cache hit rate: {cache_hit_rate:.2}%");

    // Also get disk usage stats for comparison/debugging
    let cache_stats = state
        .cache
        .get_stats()
        .await
        .map_err(|e| ApiError::ParseError(format!("Failed to get cache disk stats: {e}")))?;

    debug!(
        "Cache disk stats: {} entries, {} bytes ({:.2} MB)",
        cache_stats.total_entries,
        cache_stats.total_size_bytes,
        cache_stats.total_size_bytes as f64 / 1024.0 / 1024.0
    );

    debug!(
        "Using database size ({} bytes) instead of disk size ({} bytes) for analytics",
        db_size_bytes, cache_stats.total_size_bytes
    );

    let analytics = CacheAnalytics {
        total_packages: total_packages as i64,
        total_size_bytes: db_size_bytes,
        total_size_mb: db_size_bytes as f64 / 1024.0 / 1024.0,
        most_popular_packages: popular_packages,
        recent_packages,
        cache_hit_rate,
    };

    info!("Analytics response prepared successfully");
    Ok(Json(analytics))
}

// Cache management endpoints
#[get("/api/v1/cache/stats")]
pub async fn get_cache_stats(
    state: &State<AppState>,
) -> Result<Json<CacheStatsResponse>, ApiError> {
    let stats = state
        .cache
        .get_stats()
        .await
        .map_err(|e| ApiError::ParseError(format!("Failed to get cache stats: {e}")))?;

    let total_requests = stats.hit_count + stats.miss_count;
    let hit_rate = if total_requests > 0 {
        stats.hit_count as f64 / total_requests as f64 * 100.0
    } else {
        0.0
    };

    let response = CacheStatsResponse {
        enabled: state.config.cache_enabled,
        total_entries: stats.total_entries,
        total_size_bytes: stats.total_size_bytes,
        total_size_mb: stats.total_size_bytes as f64 / 1024.0 / 1024.0,
        hit_count: stats.hit_count,
        miss_count: stats.miss_count,
        hit_rate,
        cache_dir: state.config.cache_dir.clone(),
        ttl_hours: state.config.cache_ttl_hours,
    };

    Ok(Json(response))
}

#[delete("/api/v1/cache")]
pub async fn clear_cache(state: &State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    if !state.config.cache_enabled {
        return Err(ApiError::ParseError("Cache is disabled".to_string()));
    }

    state
        .cache
        .clear()
        .await
        .map_err(|e| ApiError::ParseError(format!("Failed to clear cache: {e}")))?;

    Ok(Json(serde_json::json!({
        "message": "Cache cleared successfully"
    })))
}

#[get("/api/v1/cache/health")]
pub async fn cache_health(state: &State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let stats = state
        .cache
        .get_stats()
        .await
        .map_err(|e| ApiError::ParseError(format!("Failed to get cache stats: {e}")))?;

    let health_status = if state.config.cache_enabled {
        "healthy"
    } else {
        "disabled"
    };

    Ok(Json(serde_json::json!({
        "status": health_status,
        "enabled": state.config.cache_enabled,
        "total_size_mb": stats.total_size_bytes as f64 / 1024.0 / 1024.0
    })))
}

// Authentication endpoints (simple login/register, not npm-specific)
#[post("/api/v1/login", data = "<login_request>")]
pub async fn login(
    login_request: Json<LoginRequest>,
    state: &State<AppState>,
) -> Result<Json<LoginResponse>, ApiError> {
    let (_user, token) =
        AuthService::authenticate_user(&state.database, login_request.into_inner())?;

    Ok(Json(LoginResponse { ok: true, token }))
}

#[post("/api/v1/register", data = "<register_request>")]
pub async fn register(
    register_request: Json<RegisterRequest>,
    state: &State<AppState>,
) -> Result<Json<NpmUserResponse>, ApiError> {
    let register_data = register_request.into_inner();

    let user = AuthService::register_user(&state.database, register_data.clone())?;

    // Create authentication token for the new user
    let login_request = LoginRequest {
        name: register_data.name.clone(),
        password: register_data.password.clone(),
    };

    let (_user, token) = AuthService::authenticate_user(&state.database, login_request)?;

    Ok(Json(NpmUserResponse {
        ok: true,
        id: user.id.to_string(),
        rev: "1-0".to_string(),
        token,
    }))
}

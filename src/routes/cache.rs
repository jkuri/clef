use crate::error::ApiError;
use crate::models::CacheStatsResponse;
use crate::state::AppState;
use rocket::serde::json::Json;
use rocket::{State, delete, get};
use serde_json;

#[get("/cache/stats")]
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

#[delete("/cache")]
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

#[get("/cache/health")]
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

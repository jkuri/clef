use rocket::{post, State};
use rocket::serde::json::Json;
use serde_json::Value;
use log::{info, error, debug, warn};
use crate::state::AppState;
use crate::error::ApiError;


#[post("/-/npm/v1/security/advisories/bulk", data = "<request>")]
pub async fn security_advisories_bulk(
    request: Result<Json<Value>, rocket::serde::json::Error<'_>>,
    state: &State<AppState>,
) -> Result<Json<Value>, ApiError> {
    info!("Security advisories bulk request received");

    let request_data = match request {
        Ok(json) => {
            debug!("Request payload: {}", serde_json::to_string_pretty(&json.0).unwrap_or_else(|_| "Invalid JSON".to_string()));
            json.0
        }
        Err(e) => {
            warn!("Failed to parse security advisories request as JSON: {}", e);
            // Return empty request if parsing fails
            serde_json::json!({})
        }
    };

    let url = format!("{}/-/npm/v1/security/advisories/bulk", state.config.upstream_registry);

    // Forward the request to the upstream npm registry
    let response = state.client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "pnrs-proxy/1.0")
        .json(&request_data)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send security advisories request to upstream: {}", e);
            ApiError::NetworkError(format!("Failed to contact upstream registry: {}", e))
        })?;

    if response.status().is_success() {
        match response.json::<Value>().await {
            Ok(json) => {
                info!("Successfully proxied security advisories request");
                debug!("Response: {}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Invalid JSON".to_string()));
                Ok(Json(json))
            }
            Err(e) => {
                error!("Failed to parse security advisories response: {}", e);
                Err(ApiError::ParseError(format!("Failed to parse upstream response: {}", e)))
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("Upstream security advisories request failed with status {}: {}", status, error_text);

        // Return an empty advisories response if upstream fails
        // This allows npm install to continue even if security checks fail
        let empty_response = serde_json::json!({});
        info!("Returning empty security advisories response due to upstream failure");
        Ok(Json(empty_response))
    }
}

// Alternative endpoint path that some npm versions might use
#[post("/-/npm/v1/security/audits/quick", data = "<request>")]
pub async fn security_audits_quick(
    request: Result<Json<Value>, rocket::serde::json::Error<'_>>,
    state: &State<AppState>,
) -> Result<Json<Value>, ApiError> {
    info!("Security audits quick request received");

    let request_data = match request {
        Ok(json) => {
            debug!("Request payload: {}", serde_json::to_string_pretty(&json.0).unwrap_or_else(|_| "Invalid JSON".to_string()));
            json.0
        }
        Err(e) => {
            warn!("Failed to parse security audits request as JSON: {}", e);
            // Return empty request if parsing fails
            serde_json::json!({})
        }
    };

    let url = format!("{}/-/npm/v1/security/audits/quick", state.config.upstream_registry);

    // Forward the request to the upstream npm registry
    let response = state.client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "pnrs-proxy/1.0")
        .json(&request_data)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send security audits request to upstream: {}", e);
            ApiError::NetworkError(format!("Failed to contact upstream registry: {}", e))
        })?;

    if response.status().is_success() {
        match response.json::<Value>().await {
            Ok(json) => {
                info!("Successfully proxied security audits request");
                debug!("Response: {}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Invalid JSON".to_string()));
                Ok(Json(json))
            }
            Err(e) => {
                error!("Failed to parse security audits response: {}", e);
                Err(ApiError::ParseError(format!("Failed to parse upstream response: {}", e)))
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("Upstream security audits request failed with status {}: {}", status, error_text);

        // Return an empty audits response if upstream fails
        let empty_response = serde_json::json!({
            "actions": [],
            "advisories": {},
            "muted": [],
            "metadata": {
                "vulnerabilities": {
                    "info": 0,
                    "low": 0,
                    "moderate": 0,
                    "high": 0,
                    "critical": 0
                },
                "dependencies": 0,
                "devDependencies": 0,
                "optionalDependencies": 0,
                "totalDependencies": 0
            }
        });
        info!("Returning empty security audits response due to upstream failure");
        Ok(Json(empty_response))
    }
}

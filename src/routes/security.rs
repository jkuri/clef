use rocket::{post, State, Data};
use rocket::serde::json::Json;
use rocket::data::ToByteUnit;
use rocket::tokio::io::AsyncReadExt;
use serde_json::Value;
use log::{info, error, debug};
use crate::state::AppState;
use crate::error::ApiError;


#[post("/-/npm/v1/security/advisories/bulk", data = "<data>")]
pub async fn security_advisories_bulk(
    data: Data<'_>,
    state: &State<AppState>,
) -> Result<Json<Value>, ApiError> {
    info!("Security advisories bulk request received");

    // Read the raw request body
    let mut body = Vec::new();
    let mut stream = data.open(2_u32.megabytes());
    stream.read_to_end(&mut body).await.map_err(|e| {
        error!("Failed to read request body: {}", e);
        ApiError::BadRequest(format!("Failed to read request body: {}", e))
    })?;

    debug!("Read {} bytes of request data", body.len());

    let url = format!("{}/-/npm/v1/security/advisories/bulk", state.config.upstream_registry);

    // Build the request with proper headers for gzip content
    let req_builder = state.client
        .post(&url)
        .header("User-Agent", "pnrs-proxy/1.0")
        .header("Content-Type", "application/json")
        .header("Content-Encoding", "gzip")
        .body(body);

    let response = req_builder.send().await.map_err(|e| {
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
#[post("/-/npm/v1/security/audits/quick", data = "<data>")]
pub async fn security_audits_quick(
    data: Data<'_>,
    state: &State<AppState>,
) -> Result<Json<Value>, ApiError> {
    info!("Security audits quick request received");

    // Read the raw request body
    let mut body = Vec::new();
    let mut stream = data.open(2_u32.megabytes());
    stream.read_to_end(&mut body).await.map_err(|e| {
        error!("Failed to read request body: {}", e);
        ApiError::BadRequest(format!("Failed to read request body: {}", e))
    })?;

    debug!("Read {} bytes of request data", body.len());

    let url = format!("{}/-/npm/v1/security/audits/quick", state.config.upstream_registry);

    // Build the request with proper headers for gzip content
    let req_builder = state.client
        .post(&url)
        .header("User-Agent", "pnrs-proxy/1.0")
        .header("Content-Type", "application/json")
        .header("Content-Encoding", "gzip")
        .body(body);

    let response = req_builder.send().await.map_err(|e| {
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

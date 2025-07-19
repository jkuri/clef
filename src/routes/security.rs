use crate::error::ApiError;
use crate::state::AppState;
use log::{debug, error, info};
use rocket::data::ToByteUnit;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Data, Request, State, post};
use serde_json::Value;

// Custom request guard to capture request headers for compression detection
pub struct RequestHeaders {
    pub content_encoding: Option<String>,
    pub user_agent: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let content_encoding = request
            .headers()
            .get_one("Content-Encoding")
            .map(|s| s.to_string());
        let user_agent = request
            .headers()
            .get_one("User-Agent")
            .map(|s| s.to_string());

        Outcome::Success(RequestHeaders {
            content_encoding,
            user_agent,
        })
    }
}

impl RequestHeaders {
    // Helper function to determine if request should be sent with gzip encoding
    fn should_use_gzip_encoding(&self) -> bool {
        debug!(
            "Checking compression for request - Content-Encoding: {:?}, User-Agent: {:?}",
            self.content_encoding, self.user_agent
        );

        // Check if the incoming request has Content-Encoding: gzip
        if let Some(content_encoding) = &self.content_encoding {
            if content_encoding.to_lowercase().contains("gzip") {
                debug!("Request has Content-Encoding: gzip, forwarding with gzip");
                return true;
            }
        }

        // Check User-Agent to determine package manager
        if let Some(user_agent) = &self.user_agent {
            let user_agent_lower = user_agent.to_lowercase();

            // pnpm and yarn typically don't use gzip for audit requests (check first)
            if user_agent_lower.contains("pnpm/") || user_agent_lower.contains("yarn/") {
                debug!("Detected pnpm/yarn client, not using gzip encoding");
                return false;
            }

            // npm typically sends gzipped requests
            if user_agent_lower.contains("npm/") {
                debug!("Detected npm client, using gzip encoding");
                return true;
            }
        }

        // Default to no gzip for unknown clients
        debug!("Unknown client or no specific encoding detected, not using gzip");
        false
    }
}

#[post("/registry/-/npm/v1/security/advisories/bulk", data = "<data>")]
pub async fn security_advisories_bulk(
    headers: RequestHeaders,
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

    let url = format!(
        "{}/-/npm/v1/security/advisories/bulk",
        state.config.upstream_registry
    );

    // Build the request with proper headers based on client type
    let mut req_builder = state
        .client
        .post(&url)
        .header("User-Agent", "pnrs-proxy/1.0")
        .header("Content-Type", "application/json");

    // Add gzip encoding if appropriate
    if headers.should_use_gzip_encoding() {
        req_builder = req_builder.header("Content-Encoding", "gzip");
    }

    let req_builder = req_builder.body(body);

    let response = req_builder.send().await.map_err(|e| {
        error!(
            "Failed to send security advisories request to upstream: {}",
            e
        );
        ApiError::NetworkError(format!("Failed to contact upstream registry: {}", e))
    })?;

    if response.status().is_success() {
        match response.json::<Value>().await {
            Ok(json) => {
                info!("Successfully proxied security advisories request");
                debug!(
                    "Response: {}",
                    serde_json::to_string_pretty(&json)
                        .unwrap_or_else(|_| "Invalid JSON".to_string())
                );
                Ok(Json(json))
            }
            Err(e) => {
                error!("Failed to parse security advisories response: {}", e);
                Err(ApiError::ParseError(format!(
                    "Failed to parse upstream response: {}",
                    e
                )))
            }
        }
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!(
            "Upstream security advisories request failed with status {}: {}",
            status, error_text
        );

        // Return an empty advisories response if upstream fails
        // This allows npm install to continue even if security checks fail
        let empty_response = serde_json::json!({});
        info!("Returning empty security advisories response due to upstream failure");
        Ok(Json(empty_response))
    }
}

// Main audit endpoint that pnpm uses
#[post("/registry/-/npm/v1/security/audits", data = "<data>")]
pub async fn security_audits(
    headers: RequestHeaders,
    data: Data<'_>,
    state: &State<AppState>,
) -> Result<Json<Value>, ApiError> {
    info!("Security audits request received");

    // Read the raw request body
    let mut body = Vec::new();
    let mut stream = data.open(2_u32.megabytes());
    stream.read_to_end(&mut body).await.map_err(|e| {
        error!("Failed to read request body: {}", e);
        ApiError::BadRequest(format!("Failed to read request body: {}", e))
    })?;

    debug!("Read {} bytes of request data", body.len());

    let url = format!(
        "{}/-/npm/v1/security/audits",
        state.config.upstream_registry
    );

    // Build the request with proper headers based on client type
    let mut req_builder = state
        .client
        .post(&url)
        .header("User-Agent", "pnrs-proxy/1.0")
        .header("Content-Type", "application/json");

    // Add gzip encoding if appropriate
    if headers.should_use_gzip_encoding() {
        req_builder = req_builder.header("Content-Encoding", "gzip");
    }

    let req_builder = req_builder.body(body);

    let response = req_builder.send().await.map_err(|e| {
        error!("Failed to send security audits request to upstream: {}", e);
        ApiError::NetworkError(format!("Failed to contact upstream registry: {}", e))
    })?;

    if response.status().is_success() {
        match response.json::<Value>().await {
            Ok(json) => {
                info!("Successfully proxied security audits request");
                debug!(
                    "Response: {}",
                    serde_json::to_string_pretty(&json)
                        .unwrap_or_else(|_| "Invalid JSON".to_string())
                );
                Ok(Json(json))
            }
            Err(e) => {
                error!("Failed to parse security audits response: {}", e);
                Err(ApiError::ParseError(format!(
                    "Failed to parse upstream response: {}",
                    e
                )))
            }
        }
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!(
            "Upstream security audits request failed with status {}: {}",
            status, error_text
        );

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

// Alternative endpoint path that some npm versions might use
#[post("/registry/-/npm/v1/security/audits/quick", data = "<data>")]
pub async fn security_audits_quick(
    headers: RequestHeaders,
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

    let url = format!(
        "{}/-/npm/v1/security/audits/quick",
        state.config.upstream_registry
    );

    // Build the request with proper headers based on client type
    let mut req_builder = state
        .client
        .post(&url)
        .header("User-Agent", "pnrs-proxy/1.0")
        .header("Content-Type", "application/json");

    // Add gzip encoding if appropriate
    if headers.should_use_gzip_encoding() {
        req_builder = req_builder.header("Content-Encoding", "gzip");
    }

    let req_builder = req_builder.body(body);

    let response = req_builder.send().await.map_err(|e| {
        error!("Failed to send security audits request to upstream: {}", e);
        ApiError::NetworkError(format!("Failed to contact upstream registry: {}", e))
    })?;

    if response.status().is_success() {
        match response.json::<Value>().await {
            Ok(json) => {
                info!("Successfully proxied security audits request");
                debug!(
                    "Response: {}",
                    serde_json::to_string_pretty(&json)
                        .unwrap_or_else(|_| "Invalid JSON".to_string())
                );
                Ok(Json(json))
            }
            Err(e) => {
                error!("Failed to parse security audits response: {}", e);
                Err(ApiError::ParseError(format!(
                    "Failed to parse upstream response: {}",
                    e
                )))
            }
        }
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!(
            "Upstream security audits request failed with status {}: {}",
            status, error_text
        );

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

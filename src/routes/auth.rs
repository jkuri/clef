use crate::error::ApiError;
use crate::models::{
    AuthenticatedUser, LoginRequest, LogoutResponse, NpmUserDocument, NpmUserResponse,
    RegisterRequest, WhoamiResponse,
};
use crate::services::AuthService;
use crate::state::AppState;

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

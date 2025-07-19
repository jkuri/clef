use rocket::serde::{Deserialize, Serialize};
use rocket::{
    State,
    http::Status,
    request::{FromRequest, Outcome, Request},
};

// Authentication request/response models
#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub ok: bool,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

// npm login uses a specific CouchDB-style user document format
#[derive(Deserialize, Debug)]
pub struct NpmUserDocument {
    pub _id: String, // format: "org.couchdb.user:username"
    pub name: String,
    pub password: String,
    pub email: Option<String>,
    pub r#type: String, // should be "user"
    pub roles: Option<Vec<String>>,
    pub date: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct NpmUserResponse {
    pub ok: bool,
    pub id: String,
    pub rev: String,
    pub token: String,
}

#[derive(Serialize, Debug)]
pub struct NpmErrorResponse {
    pub error: String,
    pub reason: String,
}

// npm whoami endpoint response
#[derive(Serialize, Debug)]
pub struct WhoamiResponse {
    pub username: String,
}

// npm logout endpoint response
#[derive(Serialize, Debug)]
pub struct LogoutResponse {
    pub ok: bool,
}

// Authentication guard for extracting user from Authorization header
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
    pub user_id: i32,
}

impl AuthenticatedUser {
    pub fn new(username: String, user_id: i32) -> Self {
        Self { username, user_id }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = crate::error::ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use crate::services::AuthService;
        use crate::state::AppState;

        let state = request.guard::<&State<AppState>>().await.unwrap();

        // Get Authorization header
        let auth_header = request.headers().get_one("Authorization");

        if let Some(auth_value) = auth_header {
            // npm sends "Bearer <token>" format
            if let Some(token) = auth_value.strip_prefix("Bearer ") {
                match AuthService::validate_token(&state.database, token) {
                    Ok(user) => Outcome::Success(AuthenticatedUser {
                        username: user.username,
                        user_id: user.id,
                    }),
                    Err(_) => Outcome::Error((
                        Status::Unauthorized,
                        crate::error::ApiError::Unauthorized("Invalid token".to_string()),
                    )),
                }
            } else {
                Outcome::Error((
                    Status::Unauthorized,
                    crate::error::ApiError::Unauthorized(
                        "Invalid authorization format".to_string(),
                    ),
                ))
            }
        } else {
            Outcome::Error((
                Status::Unauthorized,
                crate::error::ApiError::Unauthorized("Authorization header required".to_string()),
            ))
        }
    }
}

use crate::error::ApiError;
use crate::models::{LoginRequest, NewUser, NewUserToken, RegisterRequest, User, UserToken};
use crate::schema::{user_tokens, users};
use crate::services::DatabaseService;
use diesel::prelude::*;
use log::debug;

pub struct AuthService;

impl AuthService {
    pub fn register_user(db: &DatabaseService, request: RegisterRequest) -> Result<User, ApiError> {
        let mut conn = db.get_connection().map_err(|e| {
            ApiError::InternalServerError(format!("Database connection error: {e}"))
        })?;

        // Check if username already exists
        let existing_user = users::table
            .filter(users::username.eq(&request.name))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

        if existing_user.is_some() {
            return Err(ApiError::BadRequest("Username already exists".to_string()));
        }

        // Check if email already exists
        let existing_email = users::table
            .filter(users::email.eq(&request.email))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

        if existing_email.is_some() {
            return Err(ApiError::BadRequest("Email already exists".to_string()));
        }

        // Create new user
        let new_user = NewUser::new(request.name, request.email, request.password)
            .map_err(|e| ApiError::InternalServerError(format!("Password hashing error: {e}")))?;

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create user: {e}")))?;

        // Get the created user
        let user = users::table
            .filter(users::username.eq(&new_user.username))
            .first::<User>(&mut conn)
            .map_err(|e| {
                ApiError::InternalServerError(format!("Failed to retrieve created user: {e}"))
            })?;

        debug!("User registered successfully: {}", user.username);
        Ok(user)
    }

    pub fn authenticate_user(
        db: &DatabaseService,
        request: LoginRequest,
    ) -> Result<(User, String), ApiError> {
        let mut conn = db.get_connection().map_err(|e| {
            ApiError::InternalServerError(format!("Database connection error: {e}"))
        })?;

        // Find user by username
        let user = users::table
            .filter(users::username.eq(&request.name))
            .filter(users::is_active.eq(true))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?
            .ok_or_else(|| ApiError::Unauthorized("Invalid username or password".to_string()))?;

        // Verify password
        let password_valid = user.verify_password(&request.password).map_err(|e| {
            ApiError::InternalServerError(format!("Password verification error: {e}"))
        })?;

        if !password_valid {
            return Err(ApiError::Unauthorized(
                "Invalid username or password".to_string(),
            ));
        }

        // Create authentication token
        let new_token = NewUserToken::new_auth_token(user.id);
        let token_value = new_token.token.clone();

        diesel::insert_into(user_tokens::table)
            .values(&new_token)
            .execute(&mut conn)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create token: {e}")))?;

        debug!("User authenticated successfully: {}", user.username);
        Ok((user, token_value))
    }

    pub fn validate_token(db: &DatabaseService, token: &str) -> Result<User, ApiError> {
        let mut conn = db.get_connection().map_err(|e| {
            ApiError::InternalServerError(format!("Database connection error: {e}"))
        })?;

        // Find active token
        let user_token = user_tokens::table
            .filter(user_tokens::token.eq(token))
            .filter(user_tokens::is_active.eq(true))
            .first::<UserToken>(&mut conn)
            .optional()
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?
            .ok_or_else(|| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

        // Check if token is expired
        if let Some(expires_at) = user_token.expires_at {
            let now = chrono::Utc::now().naive_utc();
            if now > expires_at {
                return Err(ApiError::Unauthorized("Token expired".to_string()));
            }
        }

        // Get user
        let user = users::table
            .filter(users::id.eq(user_token.user_id))
            .filter(users::is_active.eq(true))
            .first::<User>(&mut conn)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to retrieve user: {e}")))?;

        Ok(user)
    }

    pub fn revoke_token(db: &DatabaseService, token: &str) -> Result<(), ApiError> {
        let mut conn = db.get_connection().map_err(|e| {
            ApiError::InternalServerError(format!("Database connection error: {e}"))
        })?;

        diesel::update(user_tokens::table.filter(user_tokens::token.eq(token)))
            .set(user_tokens::is_active.eq(false))
            .execute(&mut conn)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to revoke token: {e}")))?;

        debug!("Token revoked successfully");
        Ok(())
    }

    pub fn get_user_by_username(
        db: &DatabaseService,
        username: &str,
    ) -> Result<Option<User>, ApiError> {
        let mut conn = db.get_connection().map_err(|e| {
            ApiError::InternalServerError(format!("Database connection error: {e}"))
        })?;

        let user = users::table
            .filter(users::username.eq(username))
            .filter(users::is_active.eq(true))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| ApiError::InternalServerError(format!("Database query error: {e}")))?;

        Ok(user)
    }
}

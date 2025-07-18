use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::schema::{users, user_tokens};

// User authentication models
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_active: bool,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_active: bool,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub is_active: Option<bool>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = user_tokens)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserToken {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub token_type: String,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: bool,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = user_tokens)]
pub struct NewUserToken {
    pub user_id: i32,
    pub token: String,
    pub token_type: String,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: bool,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = user_tokens)]
pub struct UpdateUserToken {
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: Option<bool>,
}

impl NewUser {
    pub fn new(username: String, email: String, password: String) -> Result<Self, bcrypt::BcryptError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        let now = chrono::Utc::now().naive_utc();

        Ok(Self {
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
            is_active: true,
        })
    }
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        bcrypt::verify(password, &self.password_hash)
    }
}

impl NewUserToken {
    pub fn new_auth_token(user_id: i32) -> Self {
        let token = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc();
        let expires_at = now + chrono::Duration::days(30); // 30 days expiration

        Self {
            user_id,
            token,
            token_type: "auth".to_string(),
            created_at: now,
            expires_at: Some(expires_at),
            is_active: true,
        }
    }

    pub fn new_publish_token(user_id: i32) -> Self {
        let token = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().naive_utc();

        Self {
            user_id,
            token,
            token_type: "publish".to_string(),
            created_at: now,
            expires_at: None, // Publish tokens don't expire
            is_active: true,
        }
    }
}

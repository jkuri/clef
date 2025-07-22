use rocket::response::{Responder, Response};
use rocket::{Request, http::Status};
use std::io::Cursor;

#[derive(Debug)]
pub enum ApiError {
    UpstreamError(String),
    ParseError(String),
    NetworkError(String),
    CacheError(String),
    DatabaseError(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    InternalServerError(String),
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message) = match self {
            ApiError::UpstreamError(msg) => (Status::BadGateway, msg),
            ApiError::ParseError(msg) => (Status::BadRequest, msg),
            ApiError::NetworkError(msg) => (Status::BadGateway, msg),
            ApiError::CacheError(msg) => (Status::InternalServerError, msg),
            ApiError::DatabaseError(msg) => (Status::InternalServerError, msg),
            ApiError::BadRequest(msg) => (Status::BadRequest, msg),
            ApiError::Unauthorized(msg) => (Status::Unauthorized, msg),
            ApiError::Forbidden(msg) => (Status::Forbidden, msg),
            ApiError::NotFound(msg) => (Status::NotFound, msg),
            ApiError::Conflict(msg) => (Status::Conflict, msg),
            ApiError::InternalServerError(msg) => (Status::InternalServerError, msg),
        };

        Response::build()
            .status(status)
            .header(rocket::http::ContentType::Plain)
            .sized_body(message.len(), Cursor::new(message))
            .ok()
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::NetworkError(format!("Network error: {err}"))
    }
}

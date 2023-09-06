use actix_web::{
    http::{header::ContentType, StatusCode},
    ResponseError,
};
use chrono::Local;
use redis::RedisError;
use thiserror::Error;
use validator::ValidationErrors;

use super::srp::errors::SrpError;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Redis error {0}")]
    Redis(#[from] RedisError),
    #[error("reqwest error {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("BadRequest {0}")]
    BadRequestError(String),
    #[error("{0}")]
    Unauthenticated(String),
    #[error("Unallowable scope {0}")]
    Forbidden(String),
    #[error("Unknown information {0}")]
    NotFount(String),
    #[error("Not access {0}")]
    NotAccessible(String),
    #[error("Precondition failed {0}")]
    PreconditionFailed(String),
    #[error("Unprocessable content {0}")]
    UnprocessableEntity(String),
    // other from error
    #[error("internal config error {0}")]
    InternalConfigError(#[from] ConfigError),
    #[error("an unspecified internal error occurred {0}")]
    InternalError(#[from] anyhow::Error),
    #[error("remote error: {0}")]
    RemoteError(#[from] reqwest::Error),
    #[error("{0}")]
    HttpError(#[from] http::Error),
    #[error("srp error {0}")]
    SrpAuthError(#[from] SrpError),
}

impl From<ValidationErrors> for ApiError {
    fn from(value: ValidationErrors) -> Self {
        let mut msg = String::from("illegal arguments: ");
        for (field, error) in value.errors() {
            msg.push_str(format!("{field}: {:?}", error).as_str())
        }
        ApiError::BadRequestError(msg)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            ApiError::BadRequestError(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFount(_) => StatusCode::NOT_FOUND,
            ApiError::NotAccessible(_) => StatusCode::NOT_ACCEPTABLE,
            ApiError::PreconditionFailed(_) => StatusCode::PRECONDITION_FAILED,
            ApiError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::InternalConfigError(e) => match e {
                ConfigError::Reqwest(e) => e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                ConfigError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ApiError::SrpAuthError(e) => match e {
                SrpError::ProgressError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(format!(
                r#"{{
                "code": {},
                "msg": "{}",
                "timestamp": "{}"
            }}"#,
                self.status_code().as_str(),
                self,
                Local::now().naive_utc()
            ))
    }
}

use actix_web::{error::ResponseError, HttpResponse};
use http::StatusCode;

use crate::service::idp::IdpType;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("unauthentication: {0}")]
    UnAuthentication(&'static str),
    #[error("access_denied: {0}")]
    AccessDenied(&'static str),
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::UnAuthentication(msg) => HttpResponse::build(http::StatusCode::UNAUTHORIZED)
                .reason(msg)
                .finish(),
            AuthError::AccessDenied(msg) => HttpResponse::build(http::StatusCode::FORBIDDEN)
                .reason(msg)
                .finish(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Params {
    pub connection: IdpType,
}

pub struct Flow {
    pub params: Params,
    redirect_url: Option<String>,
    code: http::StatusCode,
    error: Option<AuthError>,
    message: Option<String>,
}

impl Flow {
    pub fn new(params: Params) -> Flow {
        Flow {
            params: params,
            redirect_url: None,
            code: StatusCode::OK,
            message: None,
            error: None,
        }
    }

    pub fn next_uri(&self) -> HttpResponse {
        HttpResponse::Found().append_header((http::header::LOCATION, self.redirect_url.as_ref().unwrap().to_string()))
        .finish()
    }
}

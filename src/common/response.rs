use std::collections::HashMap;
use actix_web::{body, http::header::ContentType, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub enum RespStatus {
    #[serde(rename = "20000")]
    Success,
    #[serde(rename = "40000")]
    BadRequest,
    #[serde(rename = "40400")]
    NotFound,
    #[serde(rename = "50000")]
    InternalServerError,
}

// 定义通用的响应结构体
#[derive(Serialize)]
pub struct Resp<T = ()> {
    code: RespStatus,
    message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl<T> Responder for Resp<T>
where
    T: Serialize,
{
    type Body = body::BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(&self)
    }
}

impl Resp<()> {
    pub fn ok() -> Resp<()> {
        Self::new(RespStatus::Success, "success".to_string(), None)
    }
}
impl<T> Resp<T> {
    pub fn new(code: RespStatus, message: String, data: Option<T>) -> Resp<T> {
        Resp {
            code: code,
            message: Some(message),
            data: data,
        }
    }

    pub fn success(data: Option<T>) -> Resp<T> {
        Self::new(RespStatus::Success, "success".to_string(), data)
    }

    pub fn failure(code: RespStatus, message: String) -> Resp<T> {
        Self::new(code, message, None)
    }
}

use actix_web::{
    cookie::CookieBuilder,
    error::ErrorUnauthorized,
    get,
    http::header,
    post,
    web::{self, Form, Query},
    HttpResponse, Responder,
};
use chrono::Duration;
use http::StatusCode;
use validator::Validate;

use crate::{
    common::{
        cache::{moka, redis::redis_setex},
        constant::AuthRequestType,
        errors::{ApiError, Result},
        oauth::AuthNCodeResponse,
        utils::gen_id, jwt,
    },
    dto::auth::{AuthError, AuthRequest, Flow},
};
#[get("/authorize")]
pub async fn query_authorize(Query(params): web::Query<AuthRequest>) -> Result<impl Responder> {
    authorize(&params).await
}

#[post("/authorize")]
pub async fn form_authorize(Form(form): web::Form<AuthRequest>) -> Result<impl Responder> {
    authorize(&form).await
}

async fn authorize(params: &AuthRequest) -> Result<impl Responder> {
    params.validate()?;
    let mut flow = Flow::new(params.clone());

    match moka::get_idp_config(&flow.request.client_id).await? {
        Some(client) => {
            flow.client_config = Some(client);
        }
        None => return Err(ApiError::ResponseError(ErrorUnauthorized("invalid_client"))),
    };
    // flow 校验
    flow.validate()?;

    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, flow.request.redirect_uri))
        .cookie(CookieBuilder::new("heliannuuthus", flow.id).finish())
        .finish())
}

async fn authorization_code(flow: &mut Flow) -> Result<AuthNCodeResponse> {
    let authorization_code = &gen_id(16);
    let result = AuthNCodeResponse::new(authorization_code, flow.request.state.clone());
    redis_setex(
        format!("forum:auth:code:{}", &flow.id).as_str(),
        authorization_code.to_string(),
        Duration::minutes(10),
    );
    Ok(result)
}

async fn exchange_token(flow: &mut Flow) {
    let client_config = flow.client_config.unwrap();
    match flow.flow_type {
        crate::common::constant::AuthRequestType::Oauth => {
        }
        crate::common::constant::AuthRequestType::Oidc => {}
    }
}
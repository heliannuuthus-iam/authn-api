use actix_web::{
    cookie::CookieBuilder,
    error::ErrorUnauthorized,
    get,
    http::header,
    post,
    web::{self, Form, Json, Query},
    HttpRequest, HttpResponse, Responder,
};
use chrono::Duration;
use http::StatusCode;
use validator::Validate;

use crate::{
    common::{
        cache::{moka, redis::redis_setex},
        errors::{ApiError, Result},
        utils::gen_id,
    },
    dto::auth::{validate_flow, AuthRequest, AuthorizationCode, Flow},
    service::auth_service,
};
#[get("/authorize")]
pub async fn query_authorize(Query(params): web::Query<AuthRequest>) -> Result<impl Responder> {
    authorize(&params).await
}

#[post("/authorize")]
pub async fn form_authorize(Form(form): web::Form<AuthRequest>) -> Result<impl Responder> {
    authorize(&form).await
}

#[get("/public-config")]
async fn oauth_login(req: HttpRequest) -> Result<impl Responder> {
    let mut flow = validate_flow(&req).await?;

    auth_service::build_connection(&mut flow).await.map(Json)
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

async fn authorization_code(flow: &mut Flow) -> Result<AuthorizationCode> {
    let result = AuthorizationCode::new(gen_id(16), flow.request.state.clone());
    redis_setex(
        format!("forum:auth:code:{}", &flow.id).as_str(),
        authorization_code.to_string(),
        Duration::minutes(10),
    )
    .await?;
    Ok(result)
}

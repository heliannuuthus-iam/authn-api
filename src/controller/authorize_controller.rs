use actix_web::{
    cookie::CookieBuilder,
    error::ErrorUnauthorized,
    get,
    http::header,
    post,
    web::{self, Form, Query},
    HttpResponse, Responder,
};
use http::StatusCode;
use validator::Validate;

use crate::{
    common::{
        cache::moka::get_client_idp_config,
        errors::{ApiError, Result},
    },
    dto::auth::{Flow, Params},
    service::auth_service,
};
#[get("/authorize")]
pub async fn query_authorize(Query(params): web::Query<Params>) -> Result<impl Responder> {
    authorize(params).await
}

#[post("/authorize")]
pub async fn form_authorize(Form(params): web::Form<Params>) -> Result<impl Responder> {
    authorize(params).await
}

async fn authorize(params: Params) -> Result<impl Responder> {
    params.validate()?;
    let mut flow = Flow::new(params);

    let flow = match get_client_idp_config(&flow.params.client_id).await? {
        Some(client) => {
            flow.client_config = Some(client);
            flow
        }
        None => return Err(ApiError::ResponseError(ErrorUnauthorized("invalid_client"))),
    };
    // 参数校验
    auth_service::validate_flow(&flow).await?;

    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, flow.params.redirect_uri))
        .cookie(CookieBuilder::new("heliannuuthus", flow.id).finish())
        .finish())
}

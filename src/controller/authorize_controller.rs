use actix_web::{
    cookie::CookieBuilder,
    get,
    http::header,
    post,
    web::{self, Form, Query},
    HttpResponse, Responder,
};
use http::StatusCode;
use validator::Validate;

use crate::{
    common::errors::Result,
    dto::auth::{Flow, Params},
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
    let flow = Flow::new(params);
    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, flow.params.redirect_uri))
        .cookie(CookieBuilder::new("heliannuuthus", flow.id).finish())
        .finish())
}

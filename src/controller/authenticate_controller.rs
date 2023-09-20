use actix_web::{
    get, post,
    web::{Form, Json, Query},
    HttpRequest, HttpResponse, Responder,
};
use http::{header, StatusCode};

use crate::{
    common::errors::Result,
    dto::{
        auth::validate_flow,
        password::{PreSrpRequest, PreSrpRespose, SrpPassword, SrpRequest},
    },
    service::{auth_service, user_service},
};

#[post("/registry")]
pub async fn registry(Json(form): Json<SrpPassword>) -> Result<impl Responder> {
    user_service::create_srp(&form).await?;
    Ok(HttpResponse::Ok().finish())
}

// commit identifier and A
#[get("/login")]
pub async fn pre_login(Query(query): Query<PreSrpRequest>) -> Result<impl Responder> {
    let (salt, b_pub) = user_service::pre_srp_login(&query.identifier, &query.a_pub).await?;
    Ok(Json(PreSrpRespose { salt, b_pub }))
}

#[post("/login")]
pub async fn form_login(Form(form): Form<SrpRequest>) -> Result<impl Responder> {
    user_service::srp_login(&form.identity, &form.proof).await?;
    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, "http://forum.heliannuuthus.com"))
        .finish())
}

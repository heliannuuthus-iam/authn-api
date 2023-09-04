use actix_web::{
    cookie::CookieBuilder,
    get,
    http::header,
    post,
    web::{self, Form, Query},
    HttpRequest, HttpResponse, Responder,
};
use http::StatusCode;
use validator::Validate;

use crate::{
    common::{errors::Result, oauth::AuthCodeResponse},
    dto::{
        auth::{validate_flow, Flow, Params},
        srp::{PreSrpRequest, PreSrpRespose, SrpRequest},
    },
    service::{
        auth_service,
        user_service::{pre_srp_login, srp_login},
    },
};
#[get("/authorize")]
pub async fn query_authorize(Query(params): web::Query<Params>) -> Result<impl Responder> {
    authorize(params).await
}

#[post("/authorize")]
pub async fn form_authorize(Form(params): web::Form<Params>) -> Result<impl Responder> {
    authorize(params).await
}

#[get("/oauth/login")]
async fn oauth_login(req: HttpRequest) -> Result<impl Responder> {
    let flow = validate_flow(&req).await?;
    Ok(HttpResponse::SeeOther()
        .append_header((
            http::header::LOCATION,
            auth_service::oauth_login(&flow).await?,
        ))
        .finish())
}

#[get("/oauth/callback/{connection}")]
pub async fn callback(
    request: HttpRequest,
    web::Query(code_resp): web::Query<AuthCodeResponse>,
) -> Result<impl Responder> {
    let mut flow = validate_flow(&request).await?;
    flow.code_resp = code_resp.into();
    auth_service::oauth_user_profile(&mut flow, request).await?;
    flow.dispatch()
}

// commit identifier and A
#[get("/login/pre")]
pub async fn pre_query_login(Query(query): web::Query<PreSrpRequest>) -> Result<impl Responder> {
    pre_login(query).await
}

#[post("/login/pre")]
pub async fn pre_form_login(web::Form(form): web::Form<PreSrpRequest>) -> Result<impl Responder> {
    pre_login(form).await
}

#[get("/login")]
pub async fn query_login(Query(query): web::Query<SrpRequest>) -> Result<impl Responder> {
    login(query).await
}

#[post("/login")]
pub async fn form_login(web::Form(form): web::Form<SrpRequest>) -> Result<impl Responder> {
    login(form).await
}

async fn authorize(params: Params) -> Result<impl Responder> {
    params.validate()?;
    let flow = Flow::new(params);
    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, flow.params.redirect_uri))
        .cookie(CookieBuilder::new("heliannuuthus", flow.id).finish())
        .finish())
}

async fn pre_login(form: PreSrpRequest) -> Result<impl Responder> {
    let (salt, b_pub) = pre_srp_login(&form.identifier, &form.a_pub).await?;
    Ok(web::Json(PreSrpRespose { salt, b_pub }))
}

async fn login(form: SrpRequest) -> Result<impl Responder> {
    srp_login(&form.identity, &form.proof).await?;

    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, "http://forum.heliannuuthus.com"))
        .finish())
}

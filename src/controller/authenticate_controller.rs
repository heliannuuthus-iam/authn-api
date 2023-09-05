use actix_web::{
    get, post,
    web::{Form, Json, Query},
    HttpRequest, HttpResponse, Responder,
};
use http::{header, StatusCode};

use crate::{
    common::{errors::Result, oauth::AuthCodeResponse},
    dto::{
        auth::{validate_flow, ChallengeRequest},
        srp::{PreSrpRequest, PreSrpRespose, SrpRequest},
    },
    service::{auth_service, user_service},
};

// commit identifier and A
#[get("/login/pre")]
pub async fn pre_query_login(Query(query): Query<PreSrpRequest>) -> Result<impl Responder> {
    pre_login(query).await
}

#[post("/login/pre")]
pub async fn pre_form_login(Form(form): Form<PreSrpRequest>) -> Result<impl Responder> {
    pre_login(form).await
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
    Query(code_resp): Query<AuthCodeResponse>,
) -> Result<impl Responder> {
    let mut flow = validate_flow(&request).await?;
    flow.code_resp = code_resp.into();
    auth_service::oauth_user_profile(&mut flow, request).await?;
    flow.dispatch()
}

#[get("/login")]
pub async fn query_login(Query(query): Query<SrpRequest>) -> Result<impl Responder> {
    login(query).await
}

#[post("/login")]
pub async fn form_login(Form(form): Form<SrpRequest>) -> Result<impl Responder> {
    login(form).await
}

#[post("/challenge")]
pub async fn challenge(Json(_c_req): Json<ChallengeRequest>) -> Result<impl Responder> {
    Ok("".to_string())
}

async fn pre_login(form: PreSrpRequest) -> Result<impl Responder> {
    let (salt, b_pub) = user_service::pre_srp_login(&form.identifier, &form.a_pub).await?;
    Ok(Json(PreSrpRespose { salt, b_pub }))
}

async fn login(form: SrpRequest) -> Result<impl Responder> {
    user_service::srp_login(&form.identity, &form.proof).await?;
    Ok(HttpResponse::build(StatusCode::MOVED_PERMANENTLY)
        .append_header((header::LOCATION, "http://forum.heliannuuthus.com"))
        .finish())
}

use crate::plugins::oauth;
use crate::plugins::{github::GitHubState, oauth::AuthRequest};
use crate::service::auth_service;
use crate::service::idp::IdpType;
use actix_web::{get, post, routes, web, HttpRequest, HttpResponse, Responder};
use http::header;
use oauth2::{AuthorizationCode, TokenResponse};

#[get("/authorize/{idp_type}")]
pub async fn authorize(idp_type: web::Path<IdpType>, request: HttpRequest) -> HttpResponse {
    auth_service::authorize(idp_type.into_inner(), request).await
}

#[routes]
#[post("/oauth/callback/{idp_type}")]
#[get("/oauth/callback/{idp_type}")]
pub async fn callback(
    idp_type: web::Path<String>,
    data: web::Data<GitHubState>,
    params: web::Query<AuthRequest>,
) -> HttpResponse {
    let token = &data
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code.clone()))
        .request_async(oauth::async_http_client)
        .await
        .expect("exchange_code failed");
    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            Gitlab user info:
            <pre>{:?}</pre>
            <a href="/">Home</a>
        </body>
    </html>"#,
        token.access_token()
    );
    HttpResponse::Ok().body(html)
}

#[post("/login")]
pub async fn login() -> impl Responder {
    "login success"
}

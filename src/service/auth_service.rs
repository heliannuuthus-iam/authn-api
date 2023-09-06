use actix_web::HttpRequest;
use anyhow::Context;
use chrono::Duration;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, TokenResponse};

use crate::{
    common::{
        cache::redis::{redis_get, redis_setex},
        errors::{ApiError, Result},
        oauth::{async_http_client, select_connection_client},
    },
    dto::{
        auth::{AuthError, Flow, FlowStage},
        user::UserProfile,
    },
    rpc::user_rpc::get_user_associations,
};

// 生成认证链接
pub async fn oauth_login(flow: &Flow) -> Result<String> {
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut client = select_connection_client(&flow.params.connection)?;
    let scopes = client.scopes();
    let (auth_url, csrf_token) = client
        .client()
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_code_challenge)
        .add_scopes(scopes)
        .url();
    // 将 pkce code 存入缓存中
    redis_setex(
        format!("forum:oauth:pkce:{}", csrf_token.secret()).as_str(),
        pkce_code_verifier.secret(),
        Duration::minutes(10),
    )
    .await?;
    Ok(auth_url.to_string())
}

// oauth callback
pub async fn oauth_user_profile(flow: &mut Flow, _request: HttpRequest) -> Result<()> {
    let mut client = select_connection_client(&flow.params.connection)?;
    let code_verifier = redis_get::<PkceCodeVerifier>(
        format!(
            "forum:oauth:pkce:{}",
            flow.code_resp.as_ref().unwrap().state
        )
        .as_str(),
    )
    .await?
    .unwrap();

    let token = client
        .client()
        .exchange_code(AuthorizationCode::new(
            flow.code_resp.as_ref().unwrap().code.clone(),
        ))
        .set_pkce_verifier(code_verifier)
        .request_async(async_http_client)
        .await
        .context("exchange code failed")?;

    // 查询
    if let Some(ref oauth_user) = client.userinfo(token.access_token().secret()).await? {
        flow.oauth_user = Some(oauth_user.clone());
    }

    match &flow.oauth_user {
        Some(oauth_user) => {
            if let Some(ref subject_profile) =
                get_user_associations(&oauth_user.openid, true).await?
            {
                flow.subject = Some(UserProfile::from(subject_profile.clone()));
                flow.associations = subject_profile.associations.clone();
                flow.stage = FlowStage::Authenticated;
            }
        }
        None => {
            flow.error = Some(AuthError::UnprocessableContent(format!(
                "oauth user profile get failed"
            )))
        }
    }

    // oauth 身份注入成功，置为 authenticating
    flow.stage = FlowStage::Authenticating;

    Ok(())
}

pub async fn validate_flow(flow: &Flow) -> Result<()> {
    let redirect_url = &flow.client_config.as_ref().unwrap().client.redirect_url;
    if !redirect_url.contains(&flow.params.redirect_uri) {
        return Err(ApiError::Unauthenticated(format!("invalid_redirect_url")));
    } else {
        Ok(())
    }
}

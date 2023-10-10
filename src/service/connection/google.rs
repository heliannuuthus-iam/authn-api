use std::collections::HashMap;

use actix_web::error::ErrorPaymentRequired;
use anyhow::Context;
use reqwest::Response;
use serde_json::Value;

use super::{IdentifierProvider, OAuthEndpoint, OAuthEndpointBuilder};
use crate::{
    common::{
        client::WEB_CLIENT,
        config::{env_var, env_var_default},
        errors::{ApiError, Result},
    },
    dto::{
        authorize::Flow,
        client::ClientIdpConfig,
        token::{GrantType, TokenResponse},
        user::IdpUser,
    },
    service::connection::{Connection, IdpType},
};

lazy_static::lazy_static!(
    pub static ref GOOGLE_CLIENT: Google = Google::default();
);

#[derive(Clone)]
pub struct Google {
    endpoints: OAuthEndpoint,
}

impl Google {
    pub async fn exchange_token(
        &self,
        code: &str,
        state: &str,
        flow: &Flow,
    ) -> Result<TokenResponse> {
        if let Some(config) = flow
            .client_idp_configs
            .as_ref()
            .unwrap()
            .configs
            .get(&self.types())
        {
            let mut form: HashMap<&str, &str> = HashMap::with_capacity(5);
            let redirect_uri = env_var_default::<String>(
                "GITHUB_REDIRECT_URL",
                "https://auth.heliannuuthus.com/api/callback/github".to_string(),
            );
            let grant_type = GrantType::AuthorizationCode.to_string();
            form.insert("client_id", &config.idp_client_id);
            form.insert("code", code);
            form.insert("grant_type", &grant_type);
            form.insert("redirect_uri", &redirect_uri);
            Ok(WEB_CLIENT
                .post(&self.endpoints.token_endpoint)
                .form(&form)
                .send()
                .await
                .and_then(Response::error_for_status)
                .with_context(|| {
                    let msg = "[google] exhange token failed ";
                    tracing::error!(msg);
                    msg
                })?
                .json::<TokenResponse>()
                .await
                .with_context(|| {
                    let msg = "[google] deserialize token response failed";
                    tracing::error!(msg);
                    msg
                })?)
        } else {
            Err(ApiError::Response(ErrorPaymentRequired(
                "connection missing",
            )))
        }
    }
}

impl Default for Google {
    fn default() -> Self {
        Self {
            endpoints: OAuthEndpointBuilder::default()
                .authorize_endpoint(env_var::<String>("GOOGLE_AUTHORIZE_ENDPOINT"))
                .token_endpoint(env_var::<String>("GOOGLE_TOKEN_ENDPOINT"))
                .server_endpoint(env_var::<String>("GOOGLE_SERVER_ENDPOINT"))
                .profile_endpoint(env_var::<String>("GOOGLE_PROFILE_ENDPOINT"))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl Connection for Google {
    async fn verify(
        &self,
        _identifier: &str,
        proof: serde_json::Value,
        state: Option<&str>,
        flow: &mut Flow,
    ) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl IdentifierProvider for Google {
    async fn authorize_link(
        &self,
        _config: &ClientIdpConfig,
        _extra: Vec<(&str, &str)>,
    ) -> Result<String> {
        Ok("".to_string())
    }

    async fn userinfo(&self, proof: &str) -> Result<Option<IdpUser>> {
        let body = WEB_CLIENT
            .get(self.endpoints.profile_endpoint.as_str())
            .bearer_auth(proof)
            .send()
            .await
            .and_then(Response::error_for_status)
            .with_context(|| {
                tracing::error!("[google] get oauth user profile failed");
                "[google] fetch user profile failed"
            })?
            .json::<Value>()
            .await
            .with_context(|| {
                tracing::error!("[google] user profile serialize failed");
                "[google] user profile serialize failed"
            })?;
        let mut oauth_user = IdpUser::default();
        if let Some(email) = body["email"].as_str() {
            oauth_user.email = Some(email.to_string());
        }
        if let Some(email_verified) = body["verified_email"].as_bool() {
            oauth_user.email_verified = email_verified
        }
        if let Some(name) = body["name"].as_str() {
            oauth_user.nickname = name.to_string()
        }
        if let Some(avatar) = body["picture"].as_bool() {
            oauth_user.avatar = avatar.to_string()
        }
        oauth_user.extra = serde_json::to_string(&body).unwrap_or_default();
        Ok(Some(oauth_user))
    }

    fn types(&self) -> IdpType {
        IdpType::Google
    }
}

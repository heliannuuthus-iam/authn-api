use std::collections::HashMap;

use actix_web::error::{ErrorPreconditionRequired, ErrorUnauthorized};
use anyhow::Context;
use async_trait::async_trait;
use futures_util::{FutureExt, TryFutureExt};
use reqwest::Response;

use tracing::error;
use url::Url;

use super::OAuthEndpointBuilder;
use crate::{
    common::{
        client::WEB_CLIENT,
        config::{env_var, env_var_default},
        errors::{ApiError, Result},
    },
    dto::{authorize::Flow, client::ClientIdpConfig, token::TokenResponse, user::IdpUser},
    service::connection::{Connection, IdentifierProvider, IdpType, OAuthEndpoint},
};

lazy_static::lazy_static! {
    pub static ref GITHUB_CLIENT: GitHub = GitHub::default();
}

#[derive(Clone)]
pub struct GitHub {
    endpoints: OAuthEndpoint,
}

impl Default for GitHub {
    fn default() -> Self {
        Self {
            endpoints: OAuthEndpointBuilder::default()
                .authorize_endpoint(env_var::<String>("GITHUB_AUTHORIZE_ENDPOINT"))
                .token_endpoint(env_var::<String>("GITHUB_TOKEN_ENDPOINT"))
                .server_endpoint(env_var::<String>("GITHUB_SERVER_ENDPOINT"))
                .profile_endpoint(env_var::<String>("GITHUB_PROFILE_ENDPOINT"))
                .build()
                .unwrap(),
        }
    }
}

impl GitHub {
    async fn exchange_token(&self, code: &str, state: &str, flow: &Flow) -> Result<TokenResponse> {
        let mut form = HashMap::with_capacity(4);
        if let Some(config) = flow
            .client_idp_configs
            .as_ref()
            .unwrap()
            .configs
            .get(&self.types())
        {
            form.insert("client_id", config.idp_client_id.as_str());
            form.insert("client_secret", config.idp_client_secret.as_str());
            form.insert("code", code);
            form.insert("state", state);
            Ok(reqwest::ClientBuilder::default()
                .build()
                .unwrap()
                .post(&self.endpoints.token_endpoint)
                .form(&form)
                .send()
                .await?
                .json::<TokenResponse>()
                .await
                .with_context(|| {
                    let msg = "[github] deserialize token response failed";
                    tracing::error!(msg);
                    msg
                })?)
        } else {
            Err(ApiError::Response(ErrorPreconditionRequired(
                "connection missing",
            )))
        }
    }

    async fn fetch_profile(&self, token: &str) -> Result<Option<IdpUser>> {
        Ok(WEB_CLIENT
            .get(self.endpoints.profile_endpoint.to_string())
            .bearer_auth(token)
            .send()
            .await
            .and_then(Response::error_for_status)
            .with_context(|| {
                error!("[github] get rmeote user profile failed");
                format!("[github] get user profile failed")
            })?
            .json::<serde_json::Value>()
            .await
            .map(|v| {
                let mut oauth_user = IdpUser::default();
                if let Some(id) = v["id"].as_str() {
                    oauth_user.openid = id.to_string();
                }
                if let Some(avatar) = v["avatar"].as_str() {
                    oauth_user.avatar = avatar.to_string();
                }
                if let Some(name) = v["name"].as_str() {
                    oauth_user.nickname = name.to_string();
                }
                if let Some(name) = v["gander"].as_str() {
                    oauth_user.nickname = name.to_string();
                }
                oauth_user.extra = serde_json::to_string(&v).unwrap_or_default();
                oauth_user
            })
            .ok())
    }

    async fn fetch_email(&self, token: &str) -> Result<(String, bool)> {
        let (email, verified) = WEB_CLIENT
            .get(format!(
                "{}{}",
                self.endpoints.server_endpoint.to_string(),
                "/user/emails"
            ))
            .bearer_auth(token)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .context("get user profile failed")?
            .json::<serde_json::Value>()
            .await
            .map(|emails| {
                emails
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|email| {
                        if email["visibility"].is_string()
                            && email["visibility"].as_str().unwrap() == "private"
                        {
                            Some((
                                email.clone().to_string(),
                                email["verified"].as_bool().unwrap(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(String, bool)>>()
                    .first()
                    .unwrap()
                    .clone()
            })?;
        Ok((email, verified))
    }
}

#[async_trait::async_trait]
impl IdentifierProvider for GitHub {
    async fn authorize_link(
        &self,
        config: &ClientIdpConfig,
        extra: Vec<(&str, &str)>,
    ) -> Result<String> {
        Ok(Url::parse(&self.endpoints.authorize_endpoint)
            .map(|mut tmp| {
                let mut url_query = tmp.query_pairs_mut();
                url_query
                    .append_pair("client_id", &config.idp_client_id)
                    .append_pair(
                        "redirect_uri",
                        env_var_default::<String>(
                            "GITHUB_REDIRECT_URL",
                            "https://auth.heliannuuthus.com/api/callback/github".to_string(),
                        )
                        .as_str(),
                    )
                    .append_pair("scope", "read:user user:email");
                for (name, value) in extra {
                    url_query.append_pair(name, value);
                }
                url_query.finish().to_string()
            })
            .with_context(|| {
                let msg = "github authorize url assemble failed";
                tracing::info!(msg);
                msg
            })?)
    }

    async fn userinfo(&self, proof: &str) -> Result<Option<IdpUser>> {
        let idp_user = &mut match self.fetch_profile(proof).await? {
            Some(profile) => profile,
            None => {
                return Err(ApiError::Response(ErrorPreconditionRequired(
                    "invalid github user",
                )));
            }
        };

        let (email, verified) = self.fetch_email(proof).await?;
        idp_user.email_verified = verified;
        idp_user.email = Some(email);
        Ok(Some(idp_user.clone()))
    }

    fn types(&self) -> IdpType {
        IdpType::GitHub
    }
}

#[async_trait]
impl Connection for GitHub {
    async fn verify(
        &self,
        _identifier: &str,
        proof: serde_json::Value,
        state: Option<&str>,
        flow: &mut Flow,
    ) -> Result<()> {
        if let Some(code) = proof.as_str() {
            let token_response = self
                .exchange_token(code, state.unwrap(), flow)
                .await
                .unwrap();

            if let Some(user) = self.userinfo(&token_response.access_token).await? {
                flow.idp_user = Some(user);
                Ok(())
            } else {
                return Err(ApiError::Response(ErrorUnauthorized("login_required")));
            }
        } else {
            Err(ApiError::Response(ErrorPreconditionRequired(
                "invalid proof",
            )))
        }
    }
}

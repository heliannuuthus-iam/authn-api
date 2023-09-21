use std::{
    collections::{hash_map::RandomState, HashMap},
    str::FromStr,
    sync::OnceState,
};

use actix_web::error::{ErrorBadRequest, ErrorMisdirectedRequest};
use anyhow::Context;
use async_trait::async_trait;
use chrono::Duration;
use futures_util::{FutureExt, TryFutureExt};
use http::Uri;
use openssl_sys::stat;
use reqwest::Response;
use serde_json::Value;
use tracing::error;
use url::Url;

use crate::{
    common::{
        cache::redis::redis_setex,
        client::WEB_CLIENT,
        config::env_var_default,
        errors::{ApiError, Result},
        utils::gen_random,
    },
    dto::{
        auth::{AuthError, Flow},
        user::{IdpUser, UserProfile},
    },
    service::connection::{Connection, IdentifierProvider, IdpType, OAuthEndpoint},
};

// lazy_static::lazy_static! {
//     pub static ref GITHUB_CLIENT: GitHub = GitHub::new();
// }
#[derive(Clone)]
pub struct GitHub {
    endpoints: OAuthEndpoint,
}

impl GitHub {
    pub fn new(endpoints: OAuthEndpoint) -> Self {
        Self { endpoints }
    }

    async fn exchange_token(&self, code: &str, state: &str, flow: &Flow) -> Result<String> {
        // HashMap::with_capacity(4);
        // if let Some(config) = flow
        //     .client_config
        //     .unwrap()
        //     .idp_configs
        //     .iter()
        //     .find(|&idp| self.types().eq(&idp.idp_type))
        // {
        //     form.insert("client_id", config.idp_client_id.as_str());
        //     form.insert("client_secret", config.idp_client_secret.as_str());
        //     form.insert("code", code);
        //     form.insert("state", state);
        //     &WEB_CLIENT.post(&self.endpoints.token_endpoint).form(&form)
        // } else {
        //     
        // };
        Ok("".to_string())
    }

    async fn fetch_profile(&mut self, token: &str) -> Result<Option<IdpUser>> {
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

    async fn fetch_email(&mut self, token: &str) -> Result<(Value, bool)> {
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
                            Some((email.clone(), email["verified"].as_bool().unwrap()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(Value, bool)>>()
                    .first()
                    .unwrap()
                    .clone()
            })?;
        Ok((email, verified))
    }
}

#[async_trait]
impl IdentifierProvider for GitHub {
    type Type = IdpType;

    async fn authorize(&self, flow: &mut Flow) -> Result<String> {
        if let Some(idp_config) = &flow
            .client_config
            .unwrap()
            .idp_configs
            .iter()
            .filter(|&idp| self.types().eq(&idp.idp_type))
            .next()
        {
            let (_, state) = self.pkce(flow).await?;
            Ok(Url::parse(&self.endpoints.authorize_endpoint)
                .map(|mut tmp| {
                    tmp.query_pairs_mut()
                        .append_pair("client_id", &idp_config.idp_client_id)
                        .append_pair(
                            "redirect_uri",
                            env_var_default::<String>(
                                "GITHUB_REDIRECT_URL",
                                "https://auth.heliannuuthus.com/api/callback/github".to_string(),
                            )
                            .as_str(),
                        )
                        .append_pair("scope", "read:user user:email")
                        .append_pair("state", &state)
                })
                .with_context(|| {
                    let msg = "github authorize url assemble failed";
                    tracing::info!(msg);
                    msg
                })?
                .finish()
                .to_string())
        } else {
            Err(ApiError::ResponseError(ErrorMisdirectedRequest(
                "unsupported connection",
            )))
        }
    }

    async fn userinfo(&mut self, proof: &str) -> Result<Option<IdpUser>> {
        Ok(None)
    }

    fn types(&self) -> Self::Type {
        IdpType::GitHub
    }
}

#[async_trait]
impl Connection for GitHub {
    async fn verify(
        &self,
        identifier: Option<&str>,
        proof: &str,
        state: Option<&str>,
        flow: &Flow,
    ) {
        self.exchange_token(proof, state.unwrap(), flow)
            .await
            .unwrap();
    }
}

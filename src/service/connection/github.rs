use anyhow::Context;
use async_trait::async_trait;
use futures_util::{FutureExt, TryFutureExt};
use reqwest::Response;
use serde_json::Value;
use tracing::error;
use url::Url;

use super::OAuthEndpointBuilder;
use crate::{
    common::{
        client::WEB_CLIENT,
        config::{env_var, env_var_default},
        errors::Result,
    },
    dto::{auth::Flow, client::ClientIdpConfig, user::IdpUser},
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
    async fn exchange_token(&self, _code: &str, _state: &str, _flow: &Flow) -> Result<String> {
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

    async fn userinfo(&mut self, _proof: &str) -> Result<Option<IdpUser>> {
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
        _identifier: Option<&str>,
        proof: &str,
        state: Option<&str>,
        flow: &Flow,
    ) {
        self.exchange_token(proof, state.unwrap(), flow)
            .await
            .unwrap();
    }
}

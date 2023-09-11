use anyhow::Context;
use async_trait::async_trait;
use futures_util::{FutureExt, TryFutureExt};
use oauth2::{basic::BasicClient, Scope};
use reqwest::Response;
use serde_json::Value;
use tracing::error;

use super::{OAuthUser, OauthClient};
use crate::common::{client::WEB_CLIENT, errors::Result};

#[derive(Clone, Default)]
pub struct GitHubClient {
    inner: Option<BasicClient>,
    server_endpoint: String,
    profile_endpoint: String,
    scopes: Vec<Scope>,
}

impl GitHubClient {
    pub fn new() -> Self {
        let mut sl = Self {
            ..Default::default()
        };
        let (client, server_endpoint, profile_endpoint, scopes) = sl.init();
        sl.inner = Some(client);
        sl.server_endpoint = server_endpoint;
        sl.profile_endpoint = profile_endpoint;
        sl.scopes = scopes;
        sl
    }

    async fn fetch_profile(&mut self, token: &str) -> Result<Option<OAuthUser>> {
        Ok(WEB_CLIENT
            .get(self.profile_endpoint.to_string())
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
                let mut oauth_user = OAuthUser::default();
                if let Some(id) = v["id"].as_str() {
                    oauth_user.openid = id.to_string();
                }
                if let Some(avatar) = v["avatar"].as_str() {
                    oauth_user.avatar = avatar.to_string();
                }
                if let Some(name) = v["name"].as_str() {
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
                self.server_endpoint.to_string(),
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
impl OauthClient for GitHubClient {
    fn kind(&self) -> String {
        String::from("GITHUB")
    }

    fn client(&mut self) -> BasicClient {
        self.inner.as_ref().unwrap().to_owned()
    }

    fn server_endpoint(&mut self) -> String {
        self.server_endpoint.clone()
    }

    fn profile_endpoint(&mut self) -> String {
        self.profile_endpoint.clone()
    }

    fn scopes(&mut self) -> Vec<Scope> {
        self.scopes.to_vec()
    }

    async fn userinfo(&mut self, token: &str) -> Result<Option<OAuthUser>> {
        self.fetch_profile(token).await
    }
}

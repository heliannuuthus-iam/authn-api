use anyhow::Context;
use async_trait::async_trait;
use futures_util::{FutureExt, TryFutureExt};
use reqwest::Response;
use serde_json::Value;
use tracing::error;

use crate::{
    common::{client::WEB_CLIENT, errors::Result},
    dto::{auth::Flow, user::UserProfile},
    service::connection::{IdentifierProvider, IdpType, OAuthEndpoint},
};

#[derive(Clone, Default)]
pub struct GitHub {
    endpoints: OAuthEndpoint,
}

impl GitHub {
    pub fn new(endpoints: OAuthEndpoint) -> Self {
        Self { endpoints }
    }

    async fn fetch_profile(&mut self, token: &str) -> Result<Option<UserProfile>> {
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
                let mut oauth_user = UserProfile::default();
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

    fn authenticate(&self, flow: &Flow) {}

    fn types(&self) -> Self::Type {
        IdpType::GitHub
    }

    fn userinfo(&self) -> String {
        todo!()
    }
}

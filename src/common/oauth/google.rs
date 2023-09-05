use anyhow::Context;
use async_trait::async_trait;
use oauth2::{basic::BasicClient, Scope};
use reqwest::Response;
use serde_json::Value;

use super::{OAuthUser, OauthClient};
use crate::common::{client::REQWEST, errors::Result};

#[derive(Clone, Default)]
pub struct GoogleClient {
    inner: Option<BasicClient>,
    pub server_endpoint: String,
    pub profile_endpoint: String,
    pub scopes: Vec<Scope>,
}

impl GoogleClient {
    pub fn new() -> Self {
        let mut sl = Self {
            ..Default::default()
        };
        let (client, server_endpoint, _profile_endpoint, scopes) = sl.init();
        sl.server_endpoint = server_endpoint;
        sl.inner = Some(client);
        sl.scopes = scopes;
        sl
    }
}

#[async_trait]
impl OauthClient for GoogleClient {
    fn kind(&self) -> String {
        String::from("GOOGLE")
    }

    fn client(&mut self) -> BasicClient {
        self.inner.as_ref().unwrap().clone()
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
        let body = REQWEST
            .get(self.profile_endpoint.as_str())
            .bearer_auth(token)
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
        let mut oauth_user = OAuthUser::default();
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
}

use anyhow::Context;
use async_trait::async_trait;
use oauth2::{basic::BasicClient, Scope};
use reqwest::Response;
use serde_json::Value;

use super::{IdentifierProvider, OAuthEndpoint};
use crate::{
    common::{client::WEB_CLIENT, errors::Result},
    dto::{
        auth::Flow,
        user::{IdpUser, UserProfile},
    },
    service::connection::IdpType,
};
lazy_static::lazy_static!(
    pub static ref GOOGLE_CLIENT: Google = Google::new();
);
#[derive(Clone, Default)]
pub struct Google {
    endpoints: OAuthEndpoint,
}

impl Google {
    pub fn new(endpoints: OAuthEndpoint) -> Self {
        Self { endpoints }
    }
}

#[async_trait::async_trait]
impl IdentifierProvider for Google {
    type Type = IdpType;

    async fn authorize(&self, flow: &mut Flow) -> String {
        todo!()
    }

    fn types(&self) -> Self::Type {
        IdpType::Google
    }

    async fn userinfo(&mut self, proof: &str) -> Result<Option<IdpUser>> {
        let body = WEB_CLIENT
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
}

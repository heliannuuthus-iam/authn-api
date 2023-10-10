use std::fmt::{Debug, Display};

use actix_web::error::ErrorBadRequest;
use chrono::Duration;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use self::{github::GITHUB_CLIENT, google::GOOGLE_CLIENT};
use crate::{
    common::{
        cache::redis::redis_setex,
        errors::{ApiError, Result},
        utils::{encode64, gen_random},
    },
    dto::{authorize::Flow, client::ClientIdpConfig, user::IdpUser},
};

pub mod github;
pub mod google;
pub mod user;

#[derive(Clone, Builder)]
pub struct OAuthEndpoint {
    pub server_endpoint: String,
    pub authorize_endpoint: String,
    pub token_endpoint: String,
    pub profile_endpoint: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, Copy, Eq, PartialEq)]
pub enum IdpType {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "google")]
    Google,
    #[default]
    User,
}

impl Display for IdpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdpType::GitHub => write!(f, "github"),
            IdpType::Google => write!(f, "google"),
            IdpType::User => write!(f, "forum"),
        }
    }
}

impl From<String> for IdpType {
    fn from(value: String) -> Self {
        if value == "github" {
            IdpType::GitHub
        } else if value == "google" {
            IdpType::Google
        } else {
            IdpType::User
        }
    }
}

pub fn select_identifier_provider(idp_type: &IdpType) -> Result<Box<dyn IdentifierProvider>> {
    match idp_type {
        IdpType::GitHub => Ok(Box::new(GITHUB_CLIENT.clone())),
        IdpType::Google => Ok(Box::new(GOOGLE_CLIENT.clone())),
        _ => Err(ApiError::Response(ErrorBadRequest(format!(
            "unknwon idp type({idp_type})"
        )))),
    }
}

#[async_trait::async_trait]
pub trait Connection {
    async fn verify(
        &self,
        identifier: &str,
        proof: serde_json::Value,
        state: Option<&str>,
        flow: &mut Flow,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait IdentifierProvider {
    async fn pkce(&self, flow: &Flow) -> Result<(String, String)> {
        let code_verifier = encode64(&pkce::code_verifier(128));
        let state = gen_random(16);
        redis_setex(
            format!("forum:oauth:pkce:{}", state).as_str(),
            &code_verifier,
            Duration::minutes(10),
        )
        .await?;
        redis_setex(
            format!("forum:oauth:flow:{}", state).as_str(),
            flow,
            Duration::minutes(10),
        )
        .await?;
        Ok((code_verifier, state))
    }
    async fn authorize_link(
        &self,
        config: &ClientIdpConfig,
        extra: Vec<(&str, &str)>,
    ) -> Result<String>;
    async fn userinfo(&self, proof: &str) -> Result<Option<IdpUser>>;
    fn types(&self) -> IdpType;
}

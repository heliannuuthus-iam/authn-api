use std::{fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::dto::auth::Flow;

pub mod github;
pub mod google;

pub trait IdentifierProvider {
    type Type;
    fn authenticate(&self, flow: &Flow);

    fn types(&self) -> Self::Type;
    fn userinfo(&self) -> String;
}

pub struct OAuthEndpoint {
    pub server_endpoint: String,
    pub authorize_endpoint: String,
    pub token_endpoint: String,
    pub profile_endpoint: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq)]
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

pub fn select_connection_client(idp_type: &IdpType) -> Result<Box<dyn IdentifierProvider>> {
    match idp_type {
        IdpType::GitHub => Ok(Box::new(GITHUB_CLIENT.clone())),
        IdpType::Google => Ok(Box::new(GOOGLE_CLIENT.clone())),
        _ => Err(ApiError::ResponseError(ErrorBadRequest(format!(
            "unknwon idp type({idp_type})"
        )))),
    }
}

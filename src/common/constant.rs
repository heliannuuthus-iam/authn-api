use std::fmt::Display;

use serde::{Deserialize, Serialize};
pub const FORUM_SERVER: &str = "forum-server";
pub const FORUM_SERVER_CLUSTER: &str = "default";

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum ChallengeType {
    Link,
    Code,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
pub enum IdpType {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "google")]
    Google,
    #[serde(skip)]
    #[default]
    Forum,
}

impl Display for IdpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdpType::GitHub => write!(f, "github"),
            IdpType::Google => write!(f, "google"),
            IdpType::Forum => write!(f, "forum"),
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
            IdpType::Forum
        }
    }
}

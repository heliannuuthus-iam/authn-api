use serde::{Deserialize, Serialize};
pub const FORUM_SERVER: &str = "forum-server";
pub const FORUM_SERVER_CLUSTER: &str = "default";
pub const OPENID_SCOPE: &str = "openid";
pub const OFFLINE_ACCESS_SCOPE: &str = "offline_access";
pub const TOKEN_ISSUER: &str = "https://auth.heliannuuthus.com/issuer/{}";
pub const CONFLICT_RESPONSE_TYPE: &[&ResponseType] = &[&ResponseType::IdToken, &ResponseType::Code];

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    Link,
    Code,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Code,
    Token,
    IdToken,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptType {
    #[default]
    None,
    Login,
    Consent,
    SelectAccount,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AuthRequestType {
    Oauth,
    Oidc,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum Gander {
    #[serde(rename = "m")]
    Male,
    #[serde(rename = "f")]
    Female,
    #[default]
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum TokenType {
    Bearer,
    Basic,
}

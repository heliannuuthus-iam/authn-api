use std::time::Duration;

use serde::{Deserialize, Serialize};

pub struct TokenRequest {
    pub grant_type: GrantType,
    pub client_id: String,
    pub code: Option<String>,
    // https://www.rfc-editor.org/rfc/rfc7636.html#section-4.1
    pub code_verifier: String,
    pub code_challenge_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scope: Vec<String>,
    pub expires_in: Duration,
    pub state: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    ClientCredentials,
    RefreshToken,
    AuthorizationCode,
}

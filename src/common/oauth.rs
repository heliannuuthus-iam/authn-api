use std::fmt::Display;

use async_trait::async_trait;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, HttpRequest, HttpResponse, RedirectUrl,
    Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use self::{github::GitHubClient, google::GoogleClient};
use super::errors::{ApiError, ConfigError, Result};
use crate::common::{client::REQWEST, config::env_var};

mod github;
mod google;

lazy_static::lazy_static! {
    pub static ref GITHUB_CLIENT: GitHubClient = GitHubClient::new();
    pub  static  ref  GOOGLE_CLIENT: GoogleClient = GoogleClient::new();
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCodeResponse {
    pub code: String,
    pub state: String,
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

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct OAuthUser {
    pub openid: String,
    pub nickname: String,
    pub avatar: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub extra: String,
}

#[async_trait]
pub trait OauthClient {
    fn kind(&self) -> String;
    fn client(&mut self) -> BasicClient;
    fn server_endpoint(&mut self) -> String;
    fn profile_endpoint(&mut self) -> String;
    fn scopes(&mut self) -> Vec<Scope>;

    async fn userinfo(&mut self, token: &str) -> Result<Option<OAuthUser>>;

    fn init(&mut self) -> (BasicClient, String, String, Vec<Scope>) {
        let client_id: ClientId = ClientId::new(env_var::<String>(
            format!("{}_CLIENT_ID", self.kind()).as_str(),
        ));

        let client_secret: ClientSecret = ClientSecret::new(env_var::<String>(
            format!("{}_CLIENT_SECRET", self.kind()).as_str(),
        ));

        let redirect_url: String =
            env_var::<String>(format!("{}_REDIRECT_URL", self.kind()).as_str());

        let server_endpoint: String =
            env_var::<String>(format!("{}_SERVER_ENDPOINT", self.kind()).as_str());

        let auth_url: AuthUrl = AuthUrl::new(env_var::<String>(
            format!("{}_AUTHORIZE_ENDPOINT", self.kind()).as_str(),
        ))
        .expect("Invalid authorization endpoint URL");

        let token_url: TokenUrl = TokenUrl::new(env_var::<String>(
            format!("{}_TOKEN_ENDPOINT", self.kind()).as_str(),
        ))
        .expect("Invalid token endpoint URL");

        let profile_endpoint =
            env_var::<String>(format!("{}_PROFILE_ENDPOINT", self.kind()).as_str());

        (
            BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
                .set_redirect_uri(RedirectUrl::new(redirect_url).expect("Invalid redirect URL")),
            server_endpoint,
            profile_endpoint,
            env_var::<String>(format!("{}_OAUTH_SCOPES", self.kind()).as_str())
                .split(',')
                .map(|s| Scope::new(s.to_string()))
                .collect::<Vec<Scope>>(),
        )
    }
}

pub async fn async_http_client(
    request: HttpRequest,
) -> std::result::Result<HttpResponse, ConfigError> {
    let mut request_builder = REQWEST
        .request(request.method, request.url.as_str())
        .body(request.body);

    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }

    let request = request_builder.build()?;
    let response = REQWEST.execute(request).await?;
    let status_code = response.status();
    let headers = response.headers().to_owned();
    let body = response.bytes().await?.to_vec();
    debug!(
        "status: {}, body: {:#?}",
        status_code,
        String::from_utf8_lossy(&body)
    );
    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}

pub fn select_connection_client(idp_type: &IdpType) -> Result<Box<dyn OauthClient>> {
    match idp_type {
        IdpType::GitHub => Ok(Box::new(GITHUB_CLIENT.clone())),
        IdpType::Google => Ok(Box::new(GOOGLE_CLIENT.clone())),
        _ => Err(ApiError::BadRequestError(format!(
            "unknwon idp type({idp_type})"
        ))),
    }
}
use crate::common::config::{OauthGithubConfig, self};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

#[derive(Clone, Debug)]
pub struct GitHubState {
    pub oauth_client: BasicClient,
    pub endpoint: String,
}


pub fn github_oauth_state(config: OauthGithubConfig) -> GitHubState {
    let github_client_id = ClientId::new(
        config
            .client_id
            .expect("Missing the GITHUB_CLIENT_ID environment variable."),
    );
    let github_client_secret = ClientSecret::new(
        config
            .client_secret
            .expect("Missing the GITHUB_CLIENT_SECRET environment variable."),
    );
    let github_redirect_url = config
        .redirect_uri
        .expect("Missing the GITHUB_REDIRECT_URL environment variable.");
    let api_server_endpoint = config
        .api_server_endpoint
        .expect("Missing the GITHUB_SERVER environment variable.");
    let oauth_endpoint = config
        .oauth_server_endpoint
        .expect("Missing the GITHUB_SERVER environment variable.");
    let auth_url = AuthUrl::new(format!("{}/authorize", oauth_endpoint))
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(format!("{}/access_token", oauth_endpoint))
        .expect("Invalid token endpoint URL");
    GitHubState {
        oauth_client: BasicClient::new(
            github_client_id,
            Some(github_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(github_redirect_url).expect("Invalid redirect URL")),
        endpoint: api_server_endpoint,
    }
}

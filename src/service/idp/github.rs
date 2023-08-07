use crate::dto::auth::Flow;
use crate::plugins::github::GitHubState;
use crate::service::idp::IdentifyProvider;
use derive_builder::Builder;
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};

#[derive(Builder)]
pub struct Github {
    state: GitHubState,
}

impl IdentifyProvider for Github {
    fn authentication(&self, flow: &Flow) {
        let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _csrf_token) = self
            .state
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read_user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();
    }

    fn login(&self, flow: &Flow) {}

    fn userinfo(&self) -> String {
        let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _csrf_token) = self
            .state
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read_user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        // HttpResponse::Found()
        //     .append_header((header::LOCATION, auth_url.to_string()))
        //     .finish()
        String::from("value")
    }
}

use lazy_static::lazy_static;
use serde::Deserialize;
use std::{fs, path::Path};
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub oauth: Option<OauthConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub ip: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct OauthConfig {
    pub github: Option<OauthGithubConfig>,
}

#[derive(Debug, Deserialize)]
pub struct OauthGithubConfig {
    pub api_server_endpoint: Option<String>,
    pub oauth_server_endpoint: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
}

pub fn global_config() -> Config {
    let config_content =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml"))
            .expect("config.toml is nonexistent");

    toml::from_str(config_content.as_str()).unwrap()
}

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::common::oauth::IdpType;

#[derive(Serialize, Deserialize)]
pub struct Client {
    pub client_id: String,
    pub name: String,
    pub logo: String,
    pub description: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientConfig {
    pub client_id: String,
    pub redirect_url: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientIdpConfig {
    pub client_id: String,
    pub idp_type: IdpType,
    pub idp_client_id: String,
    pub idp_client_secret: String,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("client_id", &self.client_id)
            .field("name", &self.name)
            .field("logo", &self.logo)
            .field("description", &self.description)
            .finish()
    }
}
impl Debug for ClientIdpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientIdpConfig")
            .field("client_id", &self.client_id)
            .field("idp_type", &self.idp_type)
            .field("idp_client_id", &self.idp_client_id)
            .finish()
    }
}

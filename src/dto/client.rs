use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{common::constant::ConnectionType, service::connection::IdpType};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ClientConfig {
    pub client_id: String,
    pub name: String,
    pub logo: String,
    pub description: String,
    pub secret: String,
    pub redirect_url: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ClientIdp {
    pub idp_type: IdpType,
    pub idp_client_id: String,
    pub idp_client_secret: String,
}
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ClientIdpConfig {
    pub client: ClientConfig,
    pub idp_configs: Vec<ClientIdp>,
}

impl Debug for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConfig")
            .field("client_id", &self.client_id)
            .field("name", &self.name)
            .field("logo", &self.logo)
            .field("description", &self.description)
            .field("redirect_url", &self.redirect_url)
            .finish()
    }
}

impl Debug for ClientIdp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientIdp")
            .field("idp_type", &self.idp_type)
            .field("idp_client_id", &self.idp_client_id)
            .finish()
    }
}

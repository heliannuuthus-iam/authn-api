use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::service::connection::IdpType;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ClientConfig {
    pub client_id: String,
    pub name: String,
    pub logo: String,
    pub description: String,
    pub secret: String,
    pub redirect_url: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientIdpConfigs {
    pub client_id: String,
    pub configs: HashMap<IdpType, ClientIdpConfig>,
}

impl ClientIdpConfigs {
    pub fn new(client_id: String, configs: HashMap<IdpType, ClientIdpConfig>) -> Self {
        Self { client_id, configs }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientIdpConfig {
    pub idp_type: IdpType,
    pub idp_client_id: String,
    pub idp_client_secret: String,
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

impl Debug for ClientIdpConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientIdp")
            .field("idp_type", &self.idp_type)
            .field("idp_client_id", &self.idp_client_id)
            .finish()
    }
}

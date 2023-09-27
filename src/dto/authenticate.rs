use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthenticateForm {
    pub identifier: String,
    pub proof: serde_json::Value,
    pub state: String,
}

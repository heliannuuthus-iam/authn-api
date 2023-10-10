use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize)]
pub struct PreSrpRequest {
    #[serde(rename = "i")]
    pub identifier: String,
    #[serde(rename = "a_pub")]
    pub a_pub: String,
}
#[derive(Deserialize, Serialize)]
pub struct PreSrpResponse {
    #[serde(rename = "s")]
    pub salt: String,
    #[serde(rename = "b_pub")]
    pub b_pub: String,
}

#[derive(Deserialize, Serialize)]
pub struct SrpRequest {
    #[serde(rename = "i")]
    pub identity: String,
    #[serde(rename = "m1")]
    pub proof: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SrpPassword {
    pub identifier: String,
    pub verifier: String,
    pub salt: String,
}

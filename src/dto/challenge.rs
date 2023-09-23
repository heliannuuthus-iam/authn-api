use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::constant::ChallengeType;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ChallengeCofig {
    pub client_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub challenge_type: ChallengeType,
}

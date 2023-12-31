use serde::{Deserialize, Serialize};

use crate::{common::constant::Gander, service::connection::IdpType};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct IdpUser {
    pub openid: String,
    pub nickname: String,
    pub avatar: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub extra: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserProfile {
    pub openid: String,
    pub nickname: String,
    pub avatar: String,
    pub gander: Gander,
    pub email: Option<String>,
}

impl From<IdpUser> for UserProfile {
    fn from(value: IdpUser) -> Self {
        UserProfile {
            openid: value.openid,
            nickname: value.nickname,
            avatar: value.avatar,
            gander: Gander::Unknown,
            email: value.email,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserAssociation {
    #[serde(rename = "idp_openid")]
    pub idp_openid: String,

    #[serde(rename = "idp_type")]
    pub idp_type: IdpType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubjectProfile {
    pub openid: String,
    pub nickname: String,
    pub avatar: String,
    pub email: Option<String>,
    pub gander: Gander,
    pub associations: Vec<UserAssociation>,
}

impl From<SubjectProfile> for UserProfile {
    fn from(value: SubjectProfile) -> Self {
        UserProfile {
            openid: value.openid,
            nickname: value.nickname,
            avatar: value.avatar,
            gander: value.gander,
            email: value.email,
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::dto::auth::Flow;
pub trait IdentifyProvider {
    fn authentication(&self, flow: &Flow);
    fn login(&self, flow: &Flow);
    fn userinfo(&self) -> String;
}

#[derive(Serialize, Deserialize)]
pub enum IdpType {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "wechat")]
    WeChat,
    #[serde(rename = "qq")]
    Tencent,
}

pub mod github;
pub mod google;
pub mod tencent;
pub mod wechat;

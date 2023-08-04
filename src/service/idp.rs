use serde::{Deserialize, Serialize};

pub trait IdentifyProvider {
    fn login(&self) -> String;
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
    QQ,
}

mod github;

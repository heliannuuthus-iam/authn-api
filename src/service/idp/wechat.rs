use crate::{dto::auth::Flow, service::idp::IdentifyProvider};

pub struct WeChat {}

impl IdentifyProvider for WeChat {
    fn login(&self, flow: &Flow) {}
    fn authentication(&self, flow: &Flow) {}
    fn userinfo(&self) -> String {
        String::from("WeChat")
    }
}

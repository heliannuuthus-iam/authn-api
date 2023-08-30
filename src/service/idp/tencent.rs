use crate::{dto::auth::Flow, service::idp::IdentifyProvider};

pub struct Tencent {}

impl IdentifyProvider for Tencent {
    fn login(&self, flow: &Flow) {}
    fn authentication(&self, flow: &Flow) {}
    fn userinfo(&self) -> String {
        String::from("Tencent")
    }
}

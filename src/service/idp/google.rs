use crate::{dto::auth::Flow, service::idp::IdentifyProvider};
pub struct Google {}

impl IdentifyProvider for Google {
    fn login(&self, flow: &Flow) {}
    fn authentication(&self, flow: &Flow) {}
    fn userinfo(&self) -> String {
        String::from("Google")
    }
}

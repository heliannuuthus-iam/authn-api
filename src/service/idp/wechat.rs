use crate::service::idp::IdentifyProvider;

pub struct WeChat {}

impl IdentifyProvider for WeChat {
    fn login(&self) -> String {
        String::from("WeChat")
    }

    fn userinfo(&self) -> String {
        String::from("WeChat")
    }

    fn authentication(&self) -> String {
        String::from("WeChat")
    }
}

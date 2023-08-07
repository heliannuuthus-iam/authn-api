use crate::service::idp::IdentifyProvider;

pub struct Tencent {}

impl IdentifyProvider for Tencent {
    fn login(&self) -> String {
        String::from("Tencent")
    }

    fn userinfo(&self) -> String {
        String::from("Tencent")
    }

    fn authentication(&self) -> String {
        String::from("Tencent")
    }
}

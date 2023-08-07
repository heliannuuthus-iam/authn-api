use crate::service::idp::IdentifyProvider;

pub struct Google {}

impl IdentifyProvider for Google {
    fn login(&self) -> String {
        String::from("Google")
    }

    fn userinfo(&self) -> String {
        String::from("Google")
    }

    fn authentication(&self) -> String {
        String::from("Google")
    }
}

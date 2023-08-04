use super::idp::{IdentifyProvider, IdpType};

pub async fn authorize(idp_type: IdpType) -> String {
    let identy_provider: dyn IdentifyProvider = match idp_type {
        IdpType::GitHub => {}
        IdpType::Google => todo!(),
        IdpType::WeChat => todo!(),
        IdpType::QQ => todo!(),
    };

    String::from("")
}

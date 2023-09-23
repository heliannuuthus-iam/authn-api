use std::collections::HashMap;

use super::connection::{self};
use crate::{common::errors::Result, dto::auth::Flow};

// 生成 idp 认证链接
pub async fn build_idp(flow: &Flow) -> Result<HashMap<String, String>> {
    let idps = &flow.client_idp_configs.as_ref().unwrap().configs;
    let mut idp_links: HashMap<String, String> = HashMap::with_capacity(idps.len());
    for idp in idps.keys() {
        if connection::select_identifier_provider(idp).ok().is_none() {
            continue;
        }
        idp_links.insert(
            idp.to_string(),
            format!("https://auth.heliannuuthus.com/api/oauth/{:?}", idp),
        );
    }
    Ok(idp_links.clone())
}

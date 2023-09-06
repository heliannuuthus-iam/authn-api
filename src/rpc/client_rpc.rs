use anyhow::Context;
use http::Method;
use reqwest::Request;

use crate::{
    common::{client::REQWEST, errors::Result, nacos::rpc, oauth::IdpType},
    dto::client::{ClientConfig, ClientIdpConfig},
};
pub async fn fetch_client_config(client_id: &str) -> Result<Option<ClientConfig>> {
    Ok(REQWEST
        .execute(Request::new(
            Method::GET,
            rpc(format!("http://forum-server/clients/{client_id}").as_str()).await?,
        ))
        .await
        .with_context(|| {
            let msg = format!("fetch client({client_id}) config failed");
            tracing::error!(msg);
            msg
        })?
        .json::<ClientConfig>()
        .await
        .with_context(|| {
            let msg = format!("fetch client({client_id}) config failed");
            tracing::error!(msg);
            msg
        })
        .ok())
}

pub async fn fetch_client_idp_config(
    client_id: &str,
    idp: Option<IdpType>,
) -> Result<Option<ClientIdpConfig>> {
    let mut request_url = format!("http://forum-server/clients/{client_id}/idps");
    if let Some(idp_type) = idp {
        request_url.push_str(format!("/{}", idp_type).as_str());
    }
    Ok(REQWEST
        .execute(Request::new(Method::GET, rpc(request_url.as_str()).await?))
        .await
        .with_context(|| {
            let msg = format!("fetch client({client_id}) idps config failed");
            tracing::error!(msg);
            msg
        })?
        .json::<ClientIdpConfig>()
        .await
        .with_context(|| {
            let msg = format!("fetch client({client_id}) idps config failed");
            tracing::error!(msg);
            msg
        })
        .ok())
}

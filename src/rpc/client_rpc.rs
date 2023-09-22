use std::collections::HashMap;

use anyhow::Context;
use http::{Method, StatusCode};
use reqwest::Request;

use crate::{
    common::{
        client::WEB_CLIENT,
        errors::Result,
        nacos::{self},
    },
    dto::{
        challenge::ChallengeCofig,
        client::{ClientConfig, ClientIdpConfig, ClientIdpConfigs},
    },
    service::connection::IdpType,
};
pub async fn fetch_client_config(client_id: &str) -> Result<Option<ClientConfig>> {
    Ok(WEB_CLIENT
        .execute(Request::new(
            Method::GET,
            nacos::rpc(format!("http://forum-server/clients/{client_id}").as_str()).await?,
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
) -> Result<Option<ClientIdpConfigs>> {
    let mut request_url = format!("http://forum-server/clients/{client_id}/idps");
    if let Some(idp_type) = idp {
        request_url.push_str(format!("/{}", idp_type).as_str());
    }
    Ok(WEB_CLIENT
        .execute(Request::new(
            Method::GET,
            nacos::rpc(request_url.as_str()).await?,
        ))
        .await
        .with_context(|| {
            let msg = format!("fetch client({client_id}) idps config failed");
            tracing::error!(msg);
            msg
        })?
        .json::<Vec<ClientIdpConfig>>()
        .await
        .with_context(|| {
            let msg = format!("fetch client({client_id}) idps config failed");
            tracing::error!(msg);
            msg
        })
        .map(|idp_configs| {
            ClientIdpConfigs::new(
                client_id.to_string(),
                idp_configs
                    .iter()
                    .map(|idps| (idps.idp_type, idps.clone()))
                    .collect::<HashMap<IdpType, ClientIdpConfig>>(),
            )
        })
        .ok())
}

pub async fn fetch_challenge_config(client_id: &str) -> Result<Option<ChallengeCofig>> {
    let resp = WEB_CLIENT
        .get(
            nacos::rpc(format!("http://forum-server/config/challenge/{client_id}").as_str())
                .await?,
        )
        .send()
        .await?;

    Ok(if StatusCode::NOT_FOUND.eq(&resp.status()) {
        None
    } else {
        Some(resp.json::<ChallengeCofig>().await?)
    })
}

use anyhow::Context;
use futures_util::TryFutureExt;
use http::StatusCode;
use reqwest::Response;


use crate::{
    common::{
        client::WEB_CLIENT,
        errors::{Result},
        nacos,
    },
    dto::password::SrpPassword,
};

pub async fn fetch_srp_password(identifier: &str) -> Result<Option<SrpPassword>> {
    let resp = WEB_CLIENT
        .get(nacos::rpc(format!("http://forum-server/password/srp/{identifier}").as_str()).await?)
        .send()
        .await?;
    if StatusCode::NOT_FOUND.eq(&resp.status()) {
        Ok(None)
    } else {
        Ok(Some(resp.json::<SrpPassword>().await.with_context(
            || {
                let msg = format!("fetch srp password failed, identifier: {identifier}");
                tracing::error!(msg);
                msg
            },
        )?))
    }
}

pub async fn save_srp_password(srp: &SrpPassword) -> Result<()> {
    WEB_CLIENT
        .post(nacos::rpc(format!("http://forum-server/password/srp").as_str()).await?)
        .form(srp)
        .send()
        .await
        .and_then(Response::error_for_status)?;

    Ok(())
}

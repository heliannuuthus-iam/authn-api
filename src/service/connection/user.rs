use actix_web::error::{ErrorUnauthorized};
use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        cache::redis::redis_get,
        errors::{ApiError, Result},
        srp::{server::SrpServerVerifier},
    },
    dto::{
        authorize::Flow,
        password::{SrpRequest},
    },
    service::{
        connection::{Connection},
    },
};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {}

#[async_trait::async_trait]
impl Connection for User {
    async fn verify(
        &self,
        identifier: &str,
        proof: serde_json::Value,
        _state: Option<&str>,
        _flow: &mut Flow,
    ) -> Result<()> {
        let sr = serde_json::from_value::<SrpRequest>(proof).map_err(|err| {
            ApiError::Response(ErrorUnauthorized(format!(
                "proof is invalid format {}",
                err
            )))
        })?;
        let server_verifier =
            redis_get::<SrpServerVerifier>(format!("forum:auth:srp:{identifier}").as_str())
                .await?
                .ok_or(ApiError::Response(ErrorUnauthorized("pre login first")))?;
        server_verifier
            .verify_client(&hex::decode(sr.proof).with_context(|| {
                tracing::error!("client m1 decode failed");
                format!("client m1 decode failed")
            })?)
            .map_err(|e| {
                tracing::error!("verify client m1 failed, {:?}", e);
                ApiError::Response(ErrorUnauthorized("verify failed"))
            })?;
        Ok(())
    }
}

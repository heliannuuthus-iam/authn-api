use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use anyhow::Context;
use chrono::Duration;
use ring::rand::{SecureRandom, SystemRandom};

use crate::{
    common::{
        cache::redis::{redis_get, redis_setex},
        errors::{ApiError, Result},
        srp::{
            groups::G_2048,
            server::{SrpServer, SrpServerVerifier},
        },
    },
    dto::password::SrpPassword,
    rpc::password_rpc,
};

pub async fn pre_srp_login(i: &str, a_pub_str: &str) -> Result<(String, String)> {
    let srp_meta = match password_rpc::fetch_srp_password(i).await? {
        Some(meta) => meta,
        None => return Err(ApiError::Response(ErrorUnauthorized("invalid_identifier"))),
    };
    let srp_server = SrpServer::new(&G_2048);
    let rng = SystemRandom::new();
    let mut b = [0u8; 64];
    rng.fill(&mut b).unwrap();
    tracing::info!("b: {}", hex::encode(b));
    let verifier = hex::decode(&srp_meta.verifier).with_context(|| {
        tracing::error!("verifier decode failed");
        format!("verifier decode failed")
    })?;
    let a_pub = hex::decode(a_pub_str).with_context(|| {
        tracing::error!("a_pub decode failed");
        format!("a_pub decode failed")
    })?;
    let server_verifier: SrpServerVerifier =
        srp_server.process_reply(&b, &verifier, &a_pub).unwrap();
    redis_setex(
        format!("forum:auth:srp:{i}").as_str(),
        server_verifier,
        Duration::minutes(1),
    )
    .await?;
    Ok((
        srp_meta.salt.clone(),
        hex::encode(srp_server.compute_public_ephemeral(&b, &verifier)),
    ))
}

pub async fn srp_login(identifier: &str, m1: &str) -> Result<()> {
    let server_verifier =
        redis_get::<SrpServerVerifier>(format!("forum:auth:srp:{identifier}").as_str())
            .await?
            .ok_or(ApiError::Response(ErrorBadRequest("pre login first")))?;
    let m1 = hex::decode(m1).with_context(|| {
        tracing::error!("client m1 decode failed");
        format!("client m1 decode failed")
    })?;
    server_verifier.verify_client(&m1).map_err(|e| {
        tracing::error!("verify client m1 failed, {:?}", e);
        ApiError::Response(ErrorUnauthorized("verify failed"))
    })?;
    Ok(())
}

pub async fn create_srp(srp: &SrpPassword) -> Result<()> {
    password_rpc::save_srp_password(srp).await?;
    Ok(())
}

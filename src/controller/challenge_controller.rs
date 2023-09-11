use actix_web::{get, post, web::Json, Responder};

use crate::{
    common::{cache::moka::CLIENT_CONFIG_CACHE, errors::Result},
    dto::auth::ChallengeRequest,
};

#[get("/challenge")]
pub async fn code_challenge(Json(cq): Json<ChallengeRequest>) -> Result<impl Responder> {
    let config = CLIENT_CONFIG_CACHE.get(&cq.client_id).await;

    Ok("".to_string())
}

#[post("/challenge")]
pub async fn challenge_continous() -> Result<impl Responder> {
    Ok("".to_string())
}

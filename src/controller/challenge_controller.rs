use actix_web::{error::ErrorBadRequest, get, post, web::Json, Responder};

use crate::{
    common::{
        cache::moka,
        errors::{ApiError, Result},
    },
    dto::auth::ChallengeRequest,
};

#[get("/challenge")]
pub async fn code_challenge(Json(cq): Json<ChallengeRequest>) -> Result<impl Responder> {
    let config = match moka::get_challenge_config(&cq.client_id).await? {
        Some(config) => config,
        None => {
            return Err(ApiError::ResponseError(ErrorBadRequest(
                "invalid challenge config",
            )))
        }
    };
    
    

    Ok("".to_string())
}

#[post("/challenge")]
pub async fn challenge_continous() -> Result<impl Responder> {
    Ok("".to_string())
}

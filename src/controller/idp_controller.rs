use actix_web::{get, web::Json, HttpRequest, Responder};

use crate::{common::errors::Result, dto::auth, service::auth_service};

#[get("/public-config")]
async fn oauth_login(req: HttpRequest) -> Result<impl Responder> {
    let flow = auth::validate_flow(&req).await?;
    auth_service::build_idp(&flow).await.map(Json)
}

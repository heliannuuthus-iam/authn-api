use std::collections::HashMap;

use actix_web::error::ErrorUnprocessableEntity;

use super::connection;
use crate::{
    common::{
        cache::redis::redis_get,
        errors::{ApiError, Result},
    },
    dto::{
        auth::{Flow, FlowStage},
        user::UserProfile,
    },
    rpc::user_rpc::get_user_associations,
};

// 生成认证链接
pub async fn build_connection(flow: &mut Flow) -> Result<HashMap<String, String>> {
    flow.client_config
        .unwrap()
        .idp_configs
        .iter()
        .map(|&idp| {
            (
                idp.idp_type.to_string(),
                connection::select_connection_client(&idp.idp_type)?.authorize(flow),
            )
        })
        .collect()
}

// oauth callback
pub async fn oauth_user_profile(flow: &mut Flow) -> Result<()> {
    let mut client = connection::select_connection_client(&flow.request.connection)?;
    let code_verifier = redis_get::<String>(
        format!(
            "forum:oauth:pkce:{}",
            flow.authorization_code
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
        )
        .as_str(),
    )
    .await?
    .unwrap();

    // 查询
    if let Some(ref oauth_user) = client.userinfo().await? {
        flow.oauth_user = Some(oauth_user.clone());
    }

    match &flow.oauth_user {
        Some(oauth_user) => {
            if let Some(ref subject_profile) =
                get_user_associations(&oauth_user.openid, true).await?
            {
                flow.subject = Some(UserProfile::from(subject_profile.clone()));
                flow.associations = subject_profile.associations.clone();
                flow.stage = FlowStage::Authenticated;
            }
        }
        None => {
            return Err(ApiError::ResponseError(ErrorUnprocessableEntity(
                "oauth user profile get failed",
            )))
        }
    }

    // oauth 身份注入成功，置为 authenticating
    flow.stage = FlowStage::Authenticating;

    Ok(())
}

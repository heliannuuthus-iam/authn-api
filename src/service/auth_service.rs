use std::collections::HashMap;

use tokio_stream::{self as stream};

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
    service::connection::Connection,
};

// 生成认证链接
pub async fn build_connection(flow: &mut Flow) -> Result<HashMap<String, Box<dyn Connection>>> {
    let idps: &Vec<crate::dto::client::ClientIdp> =
        &flow.client_config.as_ref().unwrap().idp_configs;
        
    let mut res: HashMap<String, Box<dyn Connection>> = HashMap::with_capacity(idps.len());

    for idp in idps {
        res.insert(
            idp.idp_type.to_string(),
            connection::select_connection_client(&idp.idp_type)?,
        );
    }
    Ok(res)
}

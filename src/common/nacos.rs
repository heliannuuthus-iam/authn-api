use std::sync::Arc;

use actix_web::error::ErrorNotFound;
use anyhow::Context;
use nacos_sdk::api::{
    constants,
    naming::{
        NamingChangeEvent, NamingEventListener, NamingService, NamingServiceBuilder,
        ServiceInstance,
    },
    props::ClientProps,
};
use reqwest::Url;

use super::{
    config::{env_var, env_var_default},
    errors::Result,
};
use crate::common::errors::ApiError;

lazy_static::lazy_static! {
  pub static ref NACOS_CLIENT: Box<dyn NamingService + Send + Sync> = Box::new(NamingServiceBuilder::new(
    ClientProps::new()
        .server_addr(env_var::<String>("NACOS_SERVER"))
        .namespace(env_var::<String>("NACOS_NAMESPACE"))
        .app_name(env_var::<String>("CARGO_PKG_NAME"))).build().unwrap());
}

pub struct InstanceChangeListener;

impl NamingEventListener for InstanceChangeListener {
    fn event(&self, event: std::sync::Arc<NamingChangeEvent>) {
        tracing::info!("subscriber notify event={:?}", event);
    }
}

pub async fn init_nacos() {
    if env_var_default::<bool>("NACOS_REGISTRY", true) {
        let _subscribe_ret = NACOS_CLIENT
            .subscribe(
                env!("CARGO_PKG_NAME").to_string(),
                Some(constants::DEFAULT_GROUP.to_string()),
                Vec::default(),
                Arc::new(InstanceChangeListener),
            )
            .await;

        // example naming register instances
        let _register_instance_ret = NACOS_CLIENT
            .batch_register_instance(
                env!("CARGO_PKG_NAME").to_string(),
                Some(constants::DEFAULT_GROUP.to_string()),
                vec![ServiceInstance {
                    ip: env_var("SERVER_HOST"),
                    port: env_var("SERVER_PORT"),
                    ..Default::default()
                }],
            )
            .await;
    }
}

pub async fn rpc(uri: &str) -> Result<Url> {
    let mut url = Url::parse(uri).context("rpc uri parse failed")?;
    let service_name = url.host_str().ok_or(ApiError::Response(ErrorNotFound(
        "service_name parse failed",
    )))?;
    let instant = NACOS_CLIENT
        .select_one_healthy_instance(service_name.to_string(), None, Vec::default(), true)
        .await
        .with_context(|| {
            tracing::error!("select a healthy instant failed");
            format!("{service_name} unreachable: ")
        })?;
    url.set_scheme("http").unwrap();
    url.set_host(Some(instant.ip.as_str())).unwrap();
    url.set_port(Some(instant.port as u16)).unwrap();
    Ok(url)
}

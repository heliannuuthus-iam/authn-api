use chrono::Duration;

use crate::{
    common::{config::env_var_default, errors::Result},
    dto::{
        challenge::ChallengeCofig,
        client::{ClientConfig, ClientIdpConfig},
    },
    rpc::client_rpc,
};

lazy_static::lazy_static! {

static ref CLIENT_CONFIG_CACHE: moka::future::Cache::<String, ClientConfig> = moka::future::Cache::builder()
  .name("client_config_cache")
  .time_to_live(
      Duration::seconds(env_var_default::<i64>("CACHE_EXPIRES", 600))
          .to_std()
          .unwrap(),
  )
  .build();
static ref IDP_CONFIG_CACHE: moka::future::Cache::<String, ClientIdpConfig> = moka::future::Cache::builder()
  .name("client_idp_config_cache")
  .time_to_live(
      Duration::seconds(env_var_default::<i64>("CACHE_EXPIRES", 600))
          .to_std()
          .unwrap(),
  )
  .build();
static ref CHALLENGE_CONFIG_CACHE: moka::future::Cache::<String, ChallengeCofig> = moka::future::Cache::builder()
  .name("client_challenge_config_cache")
  .time_to_live(
      Duration::seconds(env_var_default::<i64>("CACHE_EXPIRES", 600))
          .to_std()
          .unwrap(),
  )
  .build();
}

pub async fn get_client_config(client_id: &str) -> Result<Option<ClientConfig>> {
    Ok(match CLIENT_CONFIG_CACHE.get(client_id).await {
        Some(client) => Some(client),
        None => match client_rpc::fetch_client_config(client_id).await? {
            Some(ref config) => {
                CLIENT_CONFIG_CACHE
                    .insert(client_id.to_string(), config.clone())
                    .await;
                Some(config.clone())
            }
            None => None,
        },
    })
}

pub async fn get_idp_config(client_id: &str) -> Result<Option<ClientIdpConfig>> {
    Ok(match IDP_CONFIG_CACHE.get(client_id).await {
        Some(client) => Some(client),
        None => match client_rpc::fetch_client_idp_config(client_id, None).await? {
            Some(ref config) => {
                IDP_CONFIG_CACHE
                    .insert(client_id.to_string(), config.clone())
                    .await;
                Some(config.clone())
            }
            None => None,
        },
    })
}

pub async fn get_challenge_config(client_id: &str) -> Result<Option<ChallengeCofig>> {
    Ok(match CHALLENGE_CONFIG_CACHE.get(client_id).await {
        Some(config) => Some(config),
        None => match client_rpc::fetch_challenge_config(client_id).await? {
            Some(ref config) => {
                CHALLENGE_CONFIG_CACHE
                    .insert(client_id.to_string(), config.clone())
                    .await;
                Some(config.clone())
            }
            None => None,
        },
    })
}

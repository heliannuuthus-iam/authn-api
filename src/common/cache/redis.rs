use anyhow::Context;
use chrono::Duration;
use redis::{aio::Connection, Client};

use crate::common::{
    config::env_var,
    errors::{ConfigError, Result},
};

lazy_static::lazy_static! {
  pub static ref CACHE: Client = {
    Client::open(format!("redis://{}:{}", env_var::<String>("REDIS_HOST"), env_var::<u16>("REDIS_PORT")))
        .map_err(ConfigError::Redis)
        .unwrap()
  };
}

pub async fn borrow() -> std::result::Result<Connection, ConfigError> {
    Ok(CACHE.get_async_connection().await?)
}

pub async fn redis_set<T>(key: &str, value: T) -> Result<()>
where
    T: serde::Serialize,
{
    let mut conn = borrow().await?;

    redis::cmd("SET")
        .arg(key)
        .arg(
            serde_json::to_string(&value)
                .context(format!("redis execute SET serialized failed: {} ", key))?,
        )
        .query_async(&mut conn)
        .await
        .context(format!("redis SET value failed: {}", key))?;

    Ok(())
}

pub async fn redis_get<T>(key: &str) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut conn = borrow().await?;
    let value: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(&mut conn)
        .await
        .context(format!("redis execute GET faild: {}", key))?;
    match value {
        Some(v) => Ok(Some(
            serde_json::from_str::<T>(&v)
                .context(format!("redis GET serlialize failed: {}", key))?,
        )),
        None => Ok(None),
    }
}

pub async fn redis_setex<T>(key: &str, value: T, expires_in: Duration) -> Result<()>
where
    T: serde::Serialize,
{
    let mut conn = borrow().await?;
    redis::cmd("SETEX")
        .arg(key)
        .arg(expires_in.num_seconds())
        .arg(
            serde_json::to_string(&value)
                .context(format!("redis SETEX serialize failed {}", key))?,
        )
        .query_async(&mut conn)
        .await
        .context(format!("redis SETEX failed {}", key))?;
    Ok(())
}

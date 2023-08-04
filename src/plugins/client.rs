use reqwest::{Client, ClientBuilder};
use std::{ io, time::Duration};
use thiserror;
use lazy_static::lazy_static;
#[derive(Debug, thiserror::Error)]
pub enum Error<T>
where
    T: std::error::Error + 'static,
{
    #[error("request failed")]
    Reqwest(#[source] T),
    #[error("request failed")]
    Http(#[source] http::Error),
    #[error("request failed")]
    Io(#[source] io::Error),
    #[error("request failed: {}", _0)]
    Other(String),
}


lazy_static! {
    pub static ref WEB_CLIENT: Client = client();
}

pub fn client() -> Client {
    ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(12)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

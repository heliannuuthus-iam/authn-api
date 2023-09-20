use std::time::Duration;

use lazy_static::lazy_static;
use reqwest::{self, ClientBuilder};

lazy_static! {
    pub static ref WEB_CLIENT: reqwest::Client = {
        ClientBuilder::new()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(12)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap()
    };
}

use reqwest::{Client, ClientBuilder};
use std::{error, fmt, io, time::Duration};

#[derive(Debug)]
pub enum Error<T>
{
    Reqwest(T),
    Http(reqwest::Error),
    Io(io::Error),
    Other(String),
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

    }
}

impl<T> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub struct AppState {
    web_client: Client,
}

impl AppState {
    pub fn get_client(&self) -> Client {
        self.web_client
    }
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

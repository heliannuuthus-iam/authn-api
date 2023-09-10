extern crate core;

use actix_web::{App, HttpServer};
use common::config::env_var;
use dotenvy::dotenv;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use crate::common::nacos::init_nacos;

mod common;
mod controller;
mod dto;
mod rpc;
mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect(".env file not found");
    let file_appender =
        tracing_appender::rolling::hourly("./log", format!("{}.log", env!("CARGO_PKG_NAME")));
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish()
            .with(tracing_subscriber::fmt::Layer::default().with_writer(non_blocking)),
    );
    init_nacos().await;
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(controller::authorize_controller::query_authorize)
            .service(controller::authorize_controller::form_authorize)
            .service(controller::authenticate_controller::pre_form_login)
            .service(controller::authenticate_controller::pre_login)
            .service(controller::authenticate_controller::query_login)
            .service(controller::authenticate_controller::form_login)
            .service(controller::authenticate_controller::oauth_login)
            .service(controller::authenticate_controller::callback)
    })
    .bind((
        env_var::<String>("SERVER_HOST"),
        env_var::<u16>("SERVER_PORT"),
    ))?
    .run()
    .await
}

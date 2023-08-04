use actix_web::{web::{self, Data}, App, HttpServer};

mod common;
mod controller;
mod service;
mod plugins;
mod dto;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config: common::config::Config = common::config::global_config();
    let server_config = config.server.unwrap();
    let oauth_config = config.oauth.unwrap();
    let github_state = plugins::github::github_oauth_state(oauth_config.github.unwrap());
    let ip: &str = server_config.ip.as_ref().expect("Invalid ip config");
    let port = server_config.port.expect("Invalid port config");
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(github_state.clone()))
            .service(controller::auth_controller::login)
            .service(controller::auth_controller::authorize)
            .service(controller::auth_controller::callback)
    })
    .bind((ip, port))?
    .run()
    .await
}

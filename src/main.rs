use actix_web::{web, App, HttpServer};
use reqwest::Client;
mod common;
mod controller;
mod plugins;



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let state = web::Data::new(plugins::client::AppState {
        web_client: plugins::client::client(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(controller::authentication_controller::login)
    })
    .bind(("127.0.0.1", 12370))?
    .run()
    .await
}

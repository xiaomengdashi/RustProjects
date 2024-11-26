mod auth;
mod models;
mod handler;
mod route;
mod config;

use std::sync::{Arc, Mutex};

use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::models::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let shared_state = Arc::new(Mutex::new(Database::new()));

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(shared_state.clone()))
          .service(
                web::scope("/api")
                .service(
                       web::scope("/auth")
                      .service(
                           web::resource("/login").route(web::post().to(handler::login))
                       )
                      .service(
                           web::resource("/register").route(web::post().to(handler::register))
                       )
                 )
                .service(web::resource("/index").wrap(HttpAuthentication::bearer(auth::validator)).route(web::get().to(handler::index)))
         )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
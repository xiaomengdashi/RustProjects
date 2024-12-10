use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use crate::state::AppState;
use crate::handlers::sse_handler;

pub async fn run_server(state: Arc<AppState>) -> std::io::Result<()> {
    println!("Server running at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/events", web::get().to(sse_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 
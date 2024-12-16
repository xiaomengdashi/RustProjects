mod config;
mod models;
mod handlers;
mod middleware;
mod api_docs;
mod state;

use actix_web::{App, HttpServer, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;

use crate::state::AppState;
use crate::models::user::User;
use crate::middleware::auth::validator;
use crate::api_docs::openapi::ApiDoc;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let openapi = ApiDoc::openapi();

    let app_state = web::Data::new(AppState {
        users: Mutex::new(vec![
            User {
                id: 1,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            },
        ]),
        next_id: Mutex::new(2),
    });

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator);
        
        App::new()
            .app_data(app_state.clone())
            .service(handlers::auth::login)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .service(handlers::user::get_users)
                    .service(handlers::user::get_user_by_id)
                    // ... 其他路由
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

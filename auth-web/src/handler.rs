use std::sync::{Arc, Mutex};


use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::config::TokenClaims;
use crate::models::{Database, RegisterRequest, LoginRequest};


pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


pub async fn register(db: web::Data<Arc<Mutex<Database>>>, body: web::Json<RegisterRequest>) -> impl Responder {
    let mut db = db.lock().unwrap();
    if db.has_user(body.username.clone()).await {
        return HttpResponse::Conflict().body(format!("user already exists: {:?}", body.username));
    };

    let user = db.create_user(body.username.clone(), body.password.clone()).await;
    return HttpResponse::Ok().body(format!("user created: {:?}", user.username));
}

pub async fn login(
    db: web::Data<Arc<Mutex<Database>>>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let db = db.lock().unwrap();
    if let Some(user) = db.get_user(body.username.clone()).await {
        if user.verify_password(&body.password) {
            match TokenClaims::generate_jwt_token(user.id.clone()) {
                Ok(token) => return HttpResponse::Ok().json(json!({ "token": token })),
                Err(_) => return HttpResponse::InternalServerError().body("Error generating token"),
            };
        }
    }

    return HttpResponse::Unauthorized().body("Invalid username or password");
}

pub async fn logout() -> impl Responder {
    HttpResponse::Ok().body("Logged out")
}
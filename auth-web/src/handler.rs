use std::sync::{Arc, Mutex};


use actix_web::{body, web, App, HttpResponse, HttpServer, Responder};

use crate::config::TokenClaims;
use crate::models::{Database, RegisterRequest, LoginRequest, LoginResponse};


pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


pub async fn register(db: web::Data<Arc<Mutex<Database>>>, body: web::Json<RegisterRequest>) -> impl Responder {
    let mut db = db.lock().unwrap();
    println!("{}", body.username.clone());
    if db.has_user(body.username.clone()).await {
        return HttpResponse::BadRequest().body("User already exists");
    };

    let user = db.create_user(body.username.clone(), body.password.clone()).await;
    return HttpResponse::Ok().body(format!("User created: {:?}", user));
}

pub async fn login(
    db: web::Data<Arc<Mutex<Database>>>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let db = db.lock().unwrap();
    if let Some(user) = db.get_user(body.username.clone()).await {
        if user.verify_password(&body.password) {
            let token = TokenClaims::generate_jwt_token(user.id.clone());
            return HttpResponse::Ok().body(format!("Token: {:?}", token));
        }
    }

    return HttpResponse::Unauthorized().body("Invalid username or password");
}
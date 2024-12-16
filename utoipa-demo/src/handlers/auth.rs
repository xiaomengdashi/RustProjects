use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::config::JWT_SECRET;
use crate::middleware::auth::Claims;
use crate::models::user::{LoginRequest, LoginResponse};

/// Login to get JWT token
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
)]
#[post("/login")]
pub async fn login(credentials: web::Json<LoginRequest>) -> impl Responder {
    if credentials.username == "admin" && credentials.password == "password" {
        let claims = Claims {
            sub: credentials.username.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };

        match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        ) {
            Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
            Err(_) => HttpResponse::InternalServerError().json("Token generation failed"),
        }
    } else {
        HttpResponse::Unauthorized().json("Invalid credentials")
    }
} 
use actix_web::{get, post, put, web, HttpResponse, Responder};
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::state::AppState;

/// Get list of users
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
#[get("/users")]
pub async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(users.to_vec())
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = u32, Path, description = "User ID")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
#[get("/users/{id}")]
pub async fn get_user_by_id(id: web::Path<u32>, data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    if let Some(user) = users.iter().find(|u| u.id == *id) {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::NotFound().json("User not found")
    }
}

// ... 其他用户相关处理器 
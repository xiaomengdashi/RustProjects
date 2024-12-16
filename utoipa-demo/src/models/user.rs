use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
} 
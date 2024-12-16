use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

// 添加应用状态来存储数据
struct AppState {
    users: Mutex<Vec<User>>,
    next_id: Mutex<u32>,
}

// 定义API文档
#[derive(OpenApi)]
#[openapi(
    paths(
        get_users,
        get_user_by_id,
        create_user,
        delete_user,
        update_user,
        get_health
    ),
    components(
        schemas(User, CreateUserRequest, UpdateUserRequest)
    ),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "health", description = "Health check endpoints")
    )
)]
struct ApiDoc;

// 数据模型
#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize, ToSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize, ToSchema)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

// API 端点
/// Get list of users
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>)
    ),
    tag = "users"
)]
#[get("/users")]
async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(users.to_vec())
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found")
    ),
    params(
        ("id" = u32, Path, description = "User ID")
    ),
    tag = "users"
)]
#[get("/users/{id}")]
async fn get_user_by_id(id: web::Path<u32>, data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    if let Some(user) = users.iter().find(|u| u.id == *id) {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::NotFound().json("User not found")
    }
}

/// Create new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = User)
    ),
    tag = "users"
)]
#[post("/users")]
async fn create_user(
    user_req: web::Json<CreateUserRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut next_id = data.next_id.lock().unwrap();
    let new_user = User {
        id: *next_id,
        name: user_req.name.clone(),
        email: user_req.email.clone(),
    };
    *next_id += 1;

    let mut users = data.users.lock().unwrap();
    users.push(new_user.clone());
    
    HttpResponse::Created().json(new_user)
}

/// Delete user
#[utoipa::path(
    delete,
    path = "/users/{id}",
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 404, description = "User not found")
    ),
    params(
        ("id" = u32, Path, description = "User ID")
    ),
    tag = "users"
)]
#[actix_web::delete("/users/{id}")]
async fn delete_user(id: web::Path<u32>, data: web::Data<AppState>) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    if let Some(pos) = users.iter().position(|u| u.id == *id) {
        users.remove(pos);
        HttpResponse::Ok().json("User deleted successfully")
    } else {
        HttpResponse::NotFound().json("User not found")
    }
}

/// Update user
#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = User),
        (status = 404, description = "User not found")
    ),
    params(
        ("id" = u32, Path, description = "User ID")
    ),
    tag = "users"
)]
#[actix_web::put("/users/{id}")]
async fn update_user(
    id: web::Path<u32>,
    user_req: web::Json<UpdateUserRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    if let Some(user) = users.iter_mut().find(|u| u.id == *id) {
        if let Some(name) = &user_req.name {
            user.name = name.clone();
        }
        if let Some(email) = &user_req.email {
            user.email = email.clone();
        }
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::NotFound().json("User not found")
    }
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy")
    ),
    tag = "health"
)]
#[get("/health")]
async fn get_health() -> impl Responder {
    HttpResponse::Ok().json("Service is healthy")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let openapi = ApiDoc::openapi();

    // 初始化应用状态
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
        App::new()
            .app_data(app_state.clone())
            .service(get_users)
            .service(get_user_by_id)
            .service(create_user)
            .service(delete_user)
            .service(update_user)
            .service(get_health)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

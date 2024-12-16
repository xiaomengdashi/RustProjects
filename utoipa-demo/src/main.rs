use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

// 定义API文档
#[derive(OpenApi)]
#[openapi(
    paths(
        get_users,
        get_user_by_id,
        create_user,
        get_health
    ),
    components(
        schemas(User, CreateUserRequest)
    ),
    tags(
        (name = "users", description = "User management endpoints"),
        (name = "health", description = "Health check endpoints")
    )
)]
struct ApiDoc;

// 数据模型
#[derive(Serialize, Deserialize, ToSchema)]
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
async fn get_users() -> impl Responder {
    let users = vec![
        User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
        },
    ];
    HttpResponse::Ok().json(users)
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
async fn get_user_by_id(id: web::Path<u32>) -> impl Responder {
    let user = User {
        id: id.into_inner(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    HttpResponse::Ok().json(user)
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
async fn create_user(user: web::Json<CreateUserRequest>) -> impl Responder {
    let new_user = User {
        id: 42,
        name: user.name.clone(),
        email: user.email.clone(),
    };
    HttpResponse::Created().json(new_user)
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

    HttpServer::new(move || {
        App::new()
            .service(get_users)
            .service(get_user_by_id)
            .service(create_user)
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

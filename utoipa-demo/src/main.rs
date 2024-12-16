use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Error, dev::ServiceRequest};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;

// JWT密钥 - 在实际应用中应该从环境变量或配置文件中读取
const JWT_SECRET: &[u8] = b"128743271948718749!@*QWQEJJF";

// 添加认证相关的结构体
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct LoginResponse {
    token: String,
}

// 添加应用状态来存储数据
struct AppState {
    users: Mutex<Vec<User>>,
    next_id: Mutex<u32>,
}

// 定义API文档
#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        get_users,
        get_user_by_id,
        create_user,
        delete_user,
        update_user,
        get_health
    ),
    components(
        schemas(User, CreateUserRequest, UpdateUserRequest, LoginRequest, LoginResponse)
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "health", description = "Health check endpoints")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // 添加安全方案定义
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );

        // 为所有非登录路径添加安全要求
        if let Some(paths) = openapi.paths.extensions.as_mut() {
            for (path, item) in paths {
                if !path.contains("/login") {
                    if let Some(path_item) = item.as_object_mut() {
                        for (_, operation) in path_item {
                            if let Some(op) = operation.as_object_mut() {
                                op.insert("security".to_string(), serde_json::json!([{
                                    "bearer_auth": Vec::<String>::new()
                                }]));
                            }
                        }
                    }
                }
            }
        }
    }
}

// 认证中间件处理函数
async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);
    
    let token = credentials.token();
    let validation = Validation::new(Algorithm::HS256);
    
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation,
    ) {
        Ok(_claims) => Ok(req),
        Err(_) => Err((AuthenticationError::from(config).into(), req)),
    }
}

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
async fn login(credentials: web::Json<LoginRequest>) -> impl Responder {
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
async fn get_users(data: web::Data<AppState>) -> impl Responder {
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
    path = "/api/users",
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
    path = "/api/users/{id}",
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
    path = "/api/users/{id}",
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
    path = "/api/health",
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
        // 创建认证中间件
        let auth = HttpAuthentication::bearer(validator);
        
        App::new()
            .app_data(app_state.clone())
            // 登录接口不需要认证
            .service(login)
            // Swagger UI 路由放在认证中间件之外
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
            // API 路由使用认证中间件保护
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .service(get_users)
                    .service(get_user_by_id)
                    .service(create_user)
                    .service(delete_user)
                    .service(update_user)
                    .service(get_health)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

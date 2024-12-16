use utoipa::OpenApi;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest, LoginRequest, LoginResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::login,
        crate::handlers::user::get_users,
        crate::handlers::user::get_user_by_id,
        // ... 其他路径
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
pub struct ApiDoc;

pub struct SecurityAddon;

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
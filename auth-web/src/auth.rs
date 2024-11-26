use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::config::TokenClaims;


// 验证 JWT token
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    
    match TokenClaims::verify_jwt_token(&token) {
        Ok(claims) => {
            println!("Token valid, User Id: {:?}", claims.sub);
            Ok(req)
        },
        Err(err) => {
            println!("Fail to validata token: {:?}", err);
            Err((actix_web::error::ErrorUnauthorized("Invalid token"), req))
        }
    }
}
use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use crate::config::JWT_SECRET;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
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
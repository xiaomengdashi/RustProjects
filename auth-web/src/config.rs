use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, Algorithm};
use chrono::{Duration, Utc};
use serde::{Serialize, Deserialize};

// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,  // 用户id
    pub exp: usize,   // 过期时间
}

impl TokenClaims {
    // 生成一个 JWT token
    pub fn generate_jwt_token(id: String) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::HS256);
        let jwt_config = JwtConfig::new();
        let claims = TokenClaims {
            sub:id,
            exp: jwt_config.expiration(),
        };
        let token = jsonwebtoken::encode(&header, &claims, &jwt_config.encoding_key())?;
        Ok(token)
    }

    // 验证 JWT token
    pub fn verify_jwt_token(token: &str) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
        let jwt_config = JwtConfig::new();

        let mut validation = Validation::default();
        validation.validate_exp = true;
        let result = jsonwebtoken::decode::<TokenClaims>(token, &jwt_config.decoding_key(), &validation)?;
        Ok(result.claims)
    }
}

// JWT 配置
pub struct JwtConfig {
    pub secret: String,
    pub expiration: Duration,
}

impl JwtConfig {
    pub fn new() -> Self {
        Self { 
            secret: String::from("12345hellfdsa_+-=*&^%$#@!"), 
            expiration: Duration::hours(24)
        }
    }

    pub fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(self.secret.as_ref())
    }

    pub fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.secret.as_ref())
    }

    pub fn header(&self) -> Header {
        Header::new(Algorithm::HS256)
    }

    pub fn expiration(&self) -> usize {
        (Utc::now() + self.expiration).timestamp() as usize
    }
}






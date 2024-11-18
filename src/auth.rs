use actix_web::{dev::ServiceRequest, error, Error, HttpMessage};
use actix_web_httpauth::{extractors::bearer::BearerAuth};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, env, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};

use crate::db::user_exists;

// Store blacklisted tokens
static BLACKLIST: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // email
    exp: usize,   // expiration time
}

pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtAuth {
    pub fn new() -> Self {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        JwtAuth {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(&self, email: &str) -> Result<String, JwtError> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 3600; // 1 hour

        let my_claims = Claims {
            sub: email.to_string(),
            exp: expiration,
        };

        encode(&Header::default(), &my_claims, &self.encoding_key)
    }

    pub fn is_blacklisted(&self, token: &str) -> bool {
        let blacklist = BLACKLIST.lock().unwrap();
        blacklist.contains(token)
    }

    pub fn blacklist_token(&self, token: &str) {
        debug!("The following token is being blacklisted: {}", token);
        let mut blacklist = BLACKLIST.lock().unwrap();
        blacklist.insert(token.to_string());
    }

    pub fn cleanup_blacklist(&self) {
        let mut blacklist = BLACKLIST.lock().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        blacklist.retain(|token| {
            if let Ok(claims) = self.validate_token(token) {
                claims.exp > current_time
            } else {
                false
            }
        });
        debug!("Blacklist cleanup completed. Current size: {}", blacklist.len());
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        let email = &token_data.claims.sub;
        info!("validate_token email: {}", email);
        match user_exists(&email).map_err(|_| JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken))? {
            true => Ok(token_data.claims),
            false => Err(JwtError::from(jsonwebtoken::errors::ErrorKind::InvalidToken)),
        }
    }
}

pub async fn validator(
     req: ServiceRequest,
    credentials: Option<BearerAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let Some(credentials) = credentials else {
        return Err((error::ErrorUnauthorized("No bearer token provided"), req));
    };
    let jwt_auth = JwtAuth::new();
    let token = credentials.token();

    if jwt_auth.is_blacklisted(token) {
        debug!("Token is blacklisted: {}", token);
        return Err((error::ErrorUnauthorized("Token is blacklisted"), req));
    }
    match jwt_auth.validate_token(token) {
        Ok(claims) => {
            info!("JWT Validation successful!");
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            warn!("Invalid JWT token");
            Err((error::ErrorUnauthorized("Invalid token"), req))
        },
    }

}
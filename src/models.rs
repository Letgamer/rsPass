use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}};
use validator::Validate;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.security_schemes.insert(
            "jwt_auth".to_string(),
            SecurityScheme::Http(HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer) // Use HttpAuthScheme::Bearer here
                .bearer_format("JWT")
                .description(Some("Enter JWT token"))
                .build()),
        );
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct PreLoginRequest {
    #[schema(format="email")]
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[schema(format = "email")]
    #[validate(email)]
    pub email: String,
    // Set password_hash to a specific Length!
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[schema(format = "email")]
    #[validate(email)]
    pub email: String,
    // Set password_hash to a specific Length!
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangeRequest {
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DataResponse {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateRequest {
    pub encrypted_data: String,
}
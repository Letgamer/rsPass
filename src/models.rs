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
    #[validate(email, length(max = 320))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[schema(format = "email")]
    #[validate(email, length(max = 320))]
    pub email: String,
    #[validate(length(max = 1024))]
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[schema(format = "email")]
    #[validate(email, length(max = 320))]
    pub email: String,
    // Set password_hash to a specific Length!
    #[validate(length(max = 1024))]
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct ChangeRequest {
    #[validate(length(max = 1024))]
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DataResponse {
    pub encrypted_data: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateRequest {
    #[validate(length(max = 1048576))]
    pub encrypted_data: String,
}
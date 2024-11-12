use actix_web::{get, post, HttpResponse, Responder};
use utoipa::{OpenApi};

use crate::models::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        route_health,
        route_email
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "accounts", description = "Account management endpoints")
    ),
    components(
        schemas(
            PreLoginRequest
        )
    )
)]
pub struct ApiDoc;

/// Check the health status of the API
#[utoipa::path(
    responses(
        (status = 200, description = "API is healthy")
    ),
    tag = "health"
)]
#[get("/api/health")]
pub async fn route_health() -> impl Responder {
    HttpResponse::Ok().finish()
}

/// Pre-Login, checks if a account for a specific e-mail already exists
#[utoipa::path(
    request_body = PreLoginRequest,
    responses(
        (status = 200, description = "Pre-login check successful")
    ),
    tag = "accounts"
)]
#[post("/api/accounts/prelogin")]
pub async fn route_email() -> impl Responder {
    HttpResponse::Ok().finish()
}
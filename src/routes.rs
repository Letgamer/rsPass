use actix_web::{get, post, HttpResponse, Responder, web};
use utoipa::{OpenApi};
use log::{error, debug, info};
use crate::db::user_exists;

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
        (status = 200, description = "User with this email already exists"),
        (status = 404, description = "No User with this email exists")
    ),
    tag = "accounts"
)]
#[post("/api/accounts/checkmail")]
pub async fn route_email(req_body: web::Json<PreLoginRequest>) -> impl Responder {
    let email = &req_body.email;
    info!("Email extracted: {}", email);
    match user_exists(email) {
        Ok(exists) => {
            if exists {
                HttpResponse::Ok().finish() // User exists
            } else {
                HttpResponse::NotFound().finish() // User does not exist
            }
        },
        Err(e) => {
            // Step 4: Handle any errors from the database query
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
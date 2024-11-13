use actix_web::{get, post, HttpResponse, Responder, web};
use utoipa::{OpenApi};
use log::{error, debug};
use validator::Validate;
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

pub fn handle_db_error(e: impl std::fmt::Display) -> HttpResponse {
    error!("Database error: {}", e);
    HttpResponse::InternalServerError().finish()
}

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
        (status = 404, description = "No User with this email exists"),
        (status = 400, description = "Invalid email format"),
        (status = 422, description = "Invalid JSON payload")
    ),
    tag = "accounts"
)]
#[post("/api/accounts/checkmail")]
pub async fn route_email(req_body: web::Json<PreLoginRequest>) -> impl Responder {
    // Validate the email format using the validator crate
    if let Err(_) = req_body.validate() {
        error!("Validation failed for email format");
        return HttpResponse::BadRequest().finish();
    }
    let email = &req_body.email;
    debug!("Email extracted: {}", email);
    match user_exists(email) {
        Ok(exists) => {
            if exists {
                HttpResponse::Ok().finish() // User exists
            } else {
                HttpResponse::NotFound().finish() // User does not exist
            }
        },
        Err(e) => handle_db_error(e),
    }
}


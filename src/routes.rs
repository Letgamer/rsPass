use actix_web::{get, post, HttpResponse, Responder, web};
use utoipa::{OpenApi};
use log::{error, debug};
use validator::Validate;
use crate::db::user_exists;
use crate::models::*;

// API Documentation struct
#[derive(OpenApi)]
#[openapi(
    paths(route_health, route_email, route_login),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "accounts", description = "Account management endpoints")
    ),
    components(schemas(PreLoginRequest, LoginRequest))
)]
pub struct ApiDoc;

// Helper to handle common database errors
fn handle_db_error(e: impl std::fmt::Display) -> HttpResponse {
    error!("Database error: {}", e);
    HttpResponse::InternalServerError().finish()
}

// Helper to validate json format
fn validate_format<T: Validate>(req_body: &web::Json<T>) -> Result<(), HttpResponse> {
    if let Err(_) = req_body.validate() {
        error!("Validation failed for input");
        return Err(HttpResponse::BadRequest().finish());
    }
    Ok(())
}


// Health check endpoint
#[utoipa::path(
    responses((status = 200, description = "API is healthy")),
    tag = "health"
)]
#[get("/api/health")]
pub async fn route_health() -> impl Responder {
    HttpResponse::Ok().finish()
}

// Check if the email exists for pre-login
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
    if let Err(response) = validate_format(&req_body) {
        return response;
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

// Log in route, with JWT generation (to be implemented)
#[utoipa::path(
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User authenticated, JWT generated"),
        (status = 401, description = "Invalid email or password"),
        (status = 400, description = "Invalid email format"),
        (status = 422, description = "Invalid JSON payload")
    ),
    tag = "accounts"
)]
#[post("/api/login")]
pub async fn route_login(req_body: web::Json<LoginRequest>) -> impl Responder {
    if let Err(response) = validate_format(&req_body) {
        return response;
    }

    let email = &req_body.email;
    let password_hash = &req_body.password_hash;
    debug!("Login attempt for email: {}", email);

    match user_exists(email) {
        Ok(exists) => {
            if exists {
                //TODO: Add actual login logic (JWT generation)
                HttpResponse::Ok().finish() // User exists
            } else {
                HttpResponse::Unauthorized().finish() // User does not exist
            }
        },
        Err(e) => handle_db_error(e),
    }
}

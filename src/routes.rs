use actix_web::{get, post, HttpResponse, Responder, web};
use utoipa::{OpenApi};
use log::{error, debug, info};
use validator::Validate;
use crate::db::user_exists;
use crate::db::user_login;
use crate::db::user_register;
use crate::db::user_changepwd;
use crate::models::*;
use crate::auth::{JwtAuth};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use crate::auth::Claims;
use actix_web::HttpRequest;
use actix_web::HttpMessage;

// API Documentation struct
#[derive(OpenApi)]
#[openapi(
    paths(route_health, route_email, route_login, route_register, route_changepwd),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication Endpoints"),
        (name = "accounts", description = "Account management endpoints")
    ),
    components(schemas(PreLoginRequest, LoginRequest, LoginResponse, ChangeRequest)),
    modifiers(&SecurityAddon)
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
        (status = 400, description = "Invalid payload"),
        (status = 500, description = "Database Error")
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
        Ok(true) => HttpResponse::Ok().finish(), // User exists
        Ok(false) => HttpResponse::NotFound().finish(), // User does not exist
        Err(e) => handle_db_error(&e),
    }
}

// Log in route, with JWT generation (to be implemented)
#[utoipa::path(
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User authenticated, JWT generated", body=LoginResponse),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Invalid email or password"),
        (status = 404, description = "User with that email doesn't exist"),
        (status = 500, description = "Database Error or JWT Generation Error")
    ),
    tag = "auth"
)]
#[post("/api/auth/login")]
pub async fn route_login(req_body: web::Json<LoginRequest>, jwt_auth: web::Data<JwtAuth>) -> impl Responder {
    if let Err(response) = validate_format(&req_body) {
        return response;
    }

    let email = &req_body.email;
    let password_hash = &req_body.password_hash;
    debug!("Login attempt for email: {}", email);

    match user_login(email, password_hash) {
        Ok(true) => {
            match jwt_auth.generate_token(email) {
                Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
                Err(e) => {
                    error!("Failed to generate token: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Ok(false) => {
            match user_exists(email) {
                Ok(true) => HttpResponse::Unauthorized().finish(), // User exists but incorrect password
                Ok(false) => HttpResponse::NotFound().finish(), // User does not exist
                Err(e) => handle_db_error(&e),
            }
        }
        Err(e) => handle_db_error(&e),
    }
}

#[utoipa::path(
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User created and authenticated, JWT generated", body=LoginResponse),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "User already exists"),
        (status = 500, description = "Database Error or JWT Generation Error")
    ),
    tag = "auth"
)]
#[post("/api/auth/register")]
pub async fn route_register(req_body: web::Json<LoginRequest>, jwt_auth: web::Data<JwtAuth>) -> impl Responder {
    if let Err(response) = validate_format(&req_body) {
        return response;
    }

    let email = &req_body.email;
    let password_hash = &req_body.password_hash;
    debug!("Register attempt for email: {}", email);
    match user_exists(email) {
        Ok(true) => HttpResponse::Unauthorized().finish(),
        Ok(false) => {
            match user_register(email, password_hash) {
                Ok(()) => {
                    match jwt_auth.generate_token(email) {
                        Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
                        Err(e) => {
                            error!("Failed to generate token: {}", e);
                            HttpResponse::InternalServerError().finish()
                        }
                    }
                }
                Err(e) => handle_db_error(&e),
            }
        }
        Err(e) => handle_db_error(&e),
    }
}


#[utoipa::path(
    post,
    path = "/api/accounts/changepwd",
    request_body = ChangeRequest,
    responses(
        (status = 200, description = "Password changed successfully!"),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "JWT Token is invalid"),
        (status = 500, description = "Database Error or JWT Generation Error")
    ),
    tag = "accounts",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn route_changepwd(
    req: HttpRequest,
    req_body: web::Json<ChangeRequest>,
    auth: BearerAuth
) -> impl Responder {
    info!("authenticated for token: {}", auth.token().to_owned());
    if let Some(claims) = req.extensions_mut().get::<Claims>() {
        // Now you can use the claims data
        let user_email = &claims.sub;
        let password_hash = &req_body.password_hash;
        info!("JWT email provided: {}", user_email);
        match user_changepwd(user_email, password_hash){
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => handle_db_error(&e),
        }
    }
    else {
        HttpResponse::InternalServerError().finish()
    }
}
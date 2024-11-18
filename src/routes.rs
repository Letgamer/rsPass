use actix_web::{get, post, HttpMessage, HttpRequest, HttpResponse, Responder, web};
use actix_web_httpauth::{extractors::bearer::BearerAuth};
use log::{debug, error, info};
use utoipa::{OpenApi};
use validator::Validate;

use crate::db::*;
use crate::models::*;
use crate::auth::{JwtAuth, Claims};

// API Documentation struct
#[derive(OpenApi)]
#[openapi(
    paths(route_health, route_email, route_login, route_register, route_changepwd, route_logout, route_delete, route_fetch, route_update),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication Endpoints"),
        (name = "accounts", description = "Account management endpoints"),
        (name = "sync", description = "Vault synchronization endpoints")
    ),
    components(schemas(PreLoginRequest, LoginRequest, LoginResponse, ChangeRequest, UpdateRequest)),
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
#[get("/api/v1/health")]
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
#[post("/api/v1/accounts/checkmail")]
pub async fn route_email(req_body: web::Json<PreLoginRequest>) -> impl Responder {
    if let Err(response) = validate_format(&req_body) {
        return response;
    }
    
    debug!("Email check for: {}", req_body.email);
    match user_exists(&req_body.email) {
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
#[post("/api/v1/auth/login")]
pub async fn route_login(req_body: web::Json<LoginRequest>, jwt_auth: web::Data<JwtAuth>) -> impl Responder {
    if let Err(response) = validate_format(&req_body) {
        return response;
    }

    debug!("Login attempt for email: {}", &req_body.email);

    match user_login(&req_body.email, &req_body.password_hash) {
        Ok(true) => {
            match jwt_auth.generate_token(&req_body.email) {
                Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
                Err(e) => {
                    error!("Failed to generate token: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Ok(false) => {
            match user_exists(&req_body.email) {
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
#[post("/api/v1/auth/register")]
pub async fn route_register(req_body: web::Json<LoginRequest>, jwt_auth: web::Data<JwtAuth>) -> impl Responder {
    if let Err(response) = validate_format(&req_body) {
        return response;
    }

    debug!("Register attempt for email: {}", &req_body.email);
    match user_exists(&req_body.email) {
        Ok(true) => HttpResponse::Unauthorized().finish(),
        Ok(false) => {
            match user_register(&req_body.email, &req_body.password_hash) {
                Ok(()) => {
                    match jwt_auth.generate_token(&req_body.email) {
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
    path = "/api/v1/accounts/changepwd",
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
pub async fn route_changepwd(req: HttpRequest, req_body: web::Json<ChangeRequest>, auth: BearerAuth) -> impl Responder {
    debug!("authenticated for token: {}", auth.token());
    if let Some(claims) = req.extensions_mut().get::<Claims>() {
        info!("Change Password of: {}", &claims.sub);
        match user_changepwd(&claims.sub, &req_body.password_hash){
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => handle_db_error(&e),
        }
    }
    else {
        HttpResponse::Unauthorized().finish()
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/accounts/logout",
    responses(
        (status = 200, description = "Logged out successfully!"),
        (status = 401, description = "JWT Token is invalid or already blacklisted")
    ),
    tag = "accounts",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn route_logout(auth: BearerAuth, jwt_auth: web::Data<JwtAuth>) -> impl Responder {
    let token = auth.token();
    debug!("Logging with token: {}", token);

    if jwt_auth.is_blacklisted(&token) {
        return HttpResponse::Unauthorized().finish();
    }
    jwt_auth.blacklist_token(&token);
    HttpResponse::Ok().finish()
}

#[utoipa::path(
    get,
    path = "/api/v1/delete",
    responses(
        (status = 200, description = "Account deleted successfully!"),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "JWT Token is invalid"),
        (status = 500, description = "Database Error or JWT Extraction Error")
    ),
    tag = "accounts",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn route_delete(req: HttpRequest, jwt_auth: web::Data<JwtAuth>, auth: BearerAuth) -> impl Responder {
    let token = auth.token();
    debug!("Deleting account with token: {}", token);

    jwt_auth.blacklist_token(&token);
    
    if let Some(claims) = req.extensions_mut().get::<Claims>() {
        info!("Deleting account of: {}", &claims.sub);
        match user_delete(&claims.sub){
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => handle_db_error(&e),
        }
    }
    else {
        HttpResponse::Unauthorized().finish()
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/sync/fetch",
    responses(
        (status = 200, description = "Fetched User Vault"),
        (status = 401, description = "JWT Token is invalid"),
        (status = 500, description = "Database Error or JWT Extraction Error")
    ),
    tag = "sync",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn route_fetch(req: HttpRequest, auth: BearerAuth) -> impl Responder {
    let token = auth.token();
    debug!("Fetching vault with token: {}", token);
    if let Some(claims) = req.extensions_mut().get::<Claims>() {
        info!("Fetching vault of user: {}", &claims.sub);
        match data_get(&claims.sub){
            Ok(data) => HttpResponse::Ok().json(DataResponse { data }),
            Err(e) => handle_db_error(&e),
        }
    }
    else {
        HttpResponse::InternalServerError().finish()
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/sync/update",
    responses(
        (status = 200, description = "Updated User Vault"),
        (status = 401, description = "JWT Token is invalid"),
        (status = 500, description = "Database Error or JWT Extraction Error")
    ),
    tag = "sync",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn route_update(req: HttpRequest, req_body: web::Json<UpdateRequest>, auth: BearerAuth) -> impl Responder {
    let token = auth.token();
    debug!("Updating vault with token: {}", token);
    if let Some(claims) = req.extensions_mut().get::<Claims>() {
        info!("Updating vault of user: {}", &claims.sub);
        match data_update(&claims.sub, &req_body.encrypted_data){
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => handle_db_error(&e),
        }
    }
    else {
        HttpResponse::InternalServerError().finish()
    }
}
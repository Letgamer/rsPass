use backend_rspass::{
    models::*,
};
use serde_json::json;

mod common;

#[actix_rt::test]
async fn test_register_valid_credentials() {
    let (jwt_auth, db_file) = common::setup();
    let jwt_auth_clone = jwt_auth.clone();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test@example.com",
            "password_hash": "validPassword123!"
        }));

    let mut response = req.await.unwrap();
    assert!(response.status().is_success());

    let body: LoginResponse = response.json().await.unwrap();
    assert!(!body.token.is_empty());

    // Validate the JWT token
    let claims = jwt_auth_clone.validate_token(&body.token).unwrap();
    assert_eq!(claims.sub, "test@example.com");

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_invalid_email_format() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "invalid-email-format",
            "password_hash": "validPassword123!"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_empty_password() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test@example.com",
            "password_hash": ""
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_very_long_password() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let long_password = "a".repeat(1100);
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test@example.com",
            "password_hash": long_password
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_very_long_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let long_local_part: String = "a".repeat(200);
    let long_email = format!("{}@example.com", long_local_part);

    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": long_email,
            "password_hash": "validPassword123!"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_same_email_twice() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    // First registration
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "duplicate@example.com",
            "password_hash": "validPassword123!"
        }));

    let response = req.await.unwrap();
    assert!(response.status().is_success());

    // Second registration with same email
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "duplicate@example.com",
            "password_hash": "differentPassword123!"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 409); // Conflict status code

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_special_characters_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test.name+tag@example.com",
            "password_hash": "validPassword123!"
        }));

    let mut response = req.await.unwrap();
    assert!(response.status().is_success());

    let body: LoginResponse = response.json().await.unwrap();
    assert!(!body.token.is_empty());

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_unexpected_json_structure() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    // Test with missing field
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test@example.com"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    // Test with additional unexpected fields
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test@example.com",
            "password_hash": "validPassword123!",
            "unexpected_field": "value"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    // Test with wrong field names
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "mail": "test@example.com",
            "pass": "validPassword123!"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}
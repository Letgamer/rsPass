use actix_web::http::StatusCode;
use backend_rspass::models::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

mod common;

async fn login(server: &actix_test::TestServer) -> String {
    let req = server.post("/api/v1/auth/login").send_json(&json!({
        "email": "test@example.com",
        "password_hash": "hash123"
    }));

    let mut response = req.await.unwrap();
    assert!(response.status().is_success());

    let body: LoginResponse = response.json().await.unwrap();
    body.token
}

async fn register(server: &actix_test::TestServer) {
    let req = server.post("/api/v1/auth/register").send_json(&json!({
        "email": "test@example.com",
        "password_hash": "hash123"
    }));

    let response = req.await.unwrap();
    assert!(response.status().is_success());
}

async fn try_authenticated_endpoints(
    server: &actix_test::TestServer,
    token: &str,
    expected_status: StatusCode,
) {
    // Test change password
    let change_pwd = server
        .post("/api/v1/account/changepwd")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .send_json(&json!({
            "password_hash": "newhash123"
        }))
        .await
        .unwrap();
    println!("changepwd: {}: ", change_pwd.status());
    println!("token: {}: ", token);
    assert_eq!(change_pwd.status(), expected_status);

    // Test fetch vault
    let fetch = server
        .get("/api/v1/sync/fetch")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .send()
        .await
        .unwrap();
    println!("fetch: {}: ", fetch.status());
    assert_eq!(fetch.status(), expected_status);

    // Test update vault
    let update = server
        .post("/api/v1/sync/update")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .send_json(&json!({
            "encrypted_data": "encrypted123"
        }))
        .await
        .unwrap();
    println!("update: {}: ", update.status());
    assert_eq!(update.status(), expected_status);
}

#[actix_rt::test]
async fn test_valid_token_flow() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register and login
    register(&server).await;
    let token = login(&server).await;

    // Try all authenticated endpoints with valid token
    try_authenticated_endpoints(&server, &token, StatusCode::OK).await;

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_blacklisted_token() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register and login
    register(&server).await;
    let token = login(&server).await;

    // Logout to blacklist token
    let logout = server
        .get("/api/v1/account/logout")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .send()
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::OK);

    // Try all authenticated endpoints with blacklisted token
    try_authenticated_endpoints(&server, &token, StatusCode::UNAUTHORIZED).await;

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_deleted_account_token() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register and login
    register(&server).await;
    let token = login(&server).await;

    // Delete account
    let delete = server
        .get("/api/v1/account/delete")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .send()
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);

    // Try all authenticated endpoints with token of deleted account
    try_authenticated_endpoints(&server, &token, StatusCode::UNAUTHORIZED).await;

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_malformed_token() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Test with malformed token
    try_authenticated_endpoints(&server, "malformed.token.here", StatusCode::UNAUTHORIZED).await;

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_invalid_signature() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Create token with different secret
    let claims = backend_rspass::auth::Claims {
        sub: "test@example.com".to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as usize, // 1 hour from now
        nonce: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("wrong_secret".as_bytes()),
    )
    .unwrap();

    // Try all authenticated endpoints with invalid signature
    try_authenticated_endpoints(&server, &token, StatusCode::UNAUTHORIZED).await;

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_modified_claims() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register and login
    register(&server).await;

    // Create token with modified claims
    let claims = backend_rspass::auth::Claims {
        sub: "hacker@evil.com".to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600) as usize, // 1 hour from now
        nonce: Uuid::new_v4().to_string(),
    };

    let modified_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("test_secret".as_bytes()),
    )
    .unwrap();

    // Try all authenticated endpoints with modified claims
    try_authenticated_endpoints(&server, &modified_token, StatusCode::UNAUTHORIZED).await;

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_expired_token() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register and login
    register(&server).await;

    // Create an expired token (set expiration time to 1 hour ago)
    let claims = backend_rspass::auth::Claims {
        sub: "test@example.com".to_string(),
        exp: (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 3600) as usize, // expired 1 hour ago
        nonce: Uuid::new_v4().to_string(),
    };

    let expired_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("test_secret".as_bytes()),
    )
    .unwrap();

    // Try all authenticated endpoints with the expired token
    try_authenticated_endpoints(&server, &expired_token, StatusCode::UNAUTHORIZED).await;

    common::cleanup(&db_file);
}

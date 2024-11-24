use backend_rspass::models::*;
use serde_json::json;

mod common;

#[actix_rt::test]
async fn test_register_success() {
    let (jwt_auth, db_file) = common::setup();
    let jwt_auth_clone = jwt_auth.clone();
    let server = common::create_server(jwt_auth);
    let req = server.post("/api/v1/auth/register").send_json(&json!({
        "email": "test@example.com",
        "password_hash": "hash123"
    }));

    let mut response = req.await.unwrap();
    println!("Status: {}", response.status());
    assert!(response.status().is_success());

    let body: LoginResponse = response.json().await.unwrap();
    assert!(!body.token.is_empty());

    // Validate the JWT token
    let claims = jwt_auth_clone.validate_token(&body.token).unwrap();
    assert_eq!(claims.sub, "test@example.com");

    common::cleanup(&db_file);
}

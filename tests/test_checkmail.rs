use serde_json::json;

mod common;

#[actix_rt::test]
async fn test_check_existing_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "existing@example.com",
            "password_hash": "hash123"
        }));

    let response = req.await.unwrap();
    assert!(response.status().is_success());

    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "email": "existing@example.com"
        }));

    let response = req.await.unwrap();
    assert!(response.status().is_success());

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_check_nonexisting_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "email": "nonexisting@example.com"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 404);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_check_malformed_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "email": "not_an_email"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_check_empty_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "email": ""
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_check_very_long_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let long_local_part: String = "a".repeat(300);
    let long_email = format!("{}@example.com", long_local_part);

    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "email": long_email
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_check_unexpected_json_structure() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);
    
    let req = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test@example.com",
            "password_hash": "hash123"
        }));

    let response = req.await.unwrap();
    assert!(response.status().is_success());

    // Test with wrong field name
    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "mail": "test@example.com"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    // Test with additional unexpected fields
    let req = server.post("/api/v1/account/checkmail")
        .send_json(&json!({
            "email": "test@example.com",
            "extra_field": "unexpected"
        }));

    let response = req.await.unwrap();
    assert_eq!(response.status(), 400);

    common::cleanup(&db_file);
}
use backend_rspass::models::*;
use serde_json::json;
use actix_web::http::StatusCode;

mod common;

#[actix_rt::test]
async fn test_register_login_logout_flow() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register
    let register_resp = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test1@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    // Login
    let mut login_resp = server.post("/api/v1/auth/login")
        .send_json(&json!({
            "email": "test1@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_body: LoginResponse = login_resp.json().await.unwrap();

    // Logout
    let logout_resp = server.get("/api/v1/account/logout")
        .bearer_auth(&login_body.token)
        .send()
        .await
        .unwrap();
    assert_eq!(logout_resp.status(), StatusCode::OK);

    // Try to use the logged-out token
    let verify_logout = server.get("/api/v1/account/logout")
        .bearer_auth(&login_body.token)
        .send()
        .await
        .unwrap();
    assert_eq!(verify_logout.status(), StatusCode::UNAUTHORIZED);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_login_change_password_flow() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register
    let mut register_resp = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test2@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);
    let register_body: LoginResponse = register_resp.json().await.unwrap();

    // Change Password
    println!("JWT: {}", &register_body.token);
    let change_pwd_resp = server.post("/api/v1/account/changepwd")
        .bearer_auth(&register_body.token)
        .send_json(&json!({
            "password_hash": "newhash123"
        }))
        .await
        .unwrap();
    assert_eq!(change_pwd_resp.status(), StatusCode::OK);

    // Try login with old password
    let old_login_resp = server.post("/api/v1/auth/login")
        .send_json(&json!({
            "email": "test2@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(old_login_resp.status(), StatusCode::UNAUTHORIZED);

    // Login with new password
    let new_login_resp = server.post("/api/v1/auth/login")
        .send_json(&json!({
            "email": "test2@example.com",
            "password_hash": "newhash123"
        }))
        .await
        .unwrap();
    assert_eq!(new_login_resp.status(), StatusCode::OK);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_login_delete_flow() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register
    let mut register_resp = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test3@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);
    let register_body: LoginResponse = register_resp.json().await.unwrap();

    // Delete Account
    let delete_resp = server.get("/api/v1/account/delete")
        .bearer_auth(&register_body.token)
        .send()
        .await
        .unwrap();
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // Try to login after deletion
    let login_resp = server.post("/api/v1/auth/login")
        .send_json(&json!({
            "email": "test3@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(login_resp.status(), StatusCode::NOT_FOUND);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_register_duplicate_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // First registration
    let first_register = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test4@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(first_register.status(), StatusCode::OK);

    // Attempt duplicate registration
    let second_register = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test4@example.com",
            "password_hash": "different_hash"
        }))
        .await
        .unwrap();
    assert_eq!(second_register.status(), StatusCode::UNAUTHORIZED);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_login_nonexistent_email() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    let login_resp = server.post("/api/v1/auth/login")
        .send_json(&json!({
            "email": "nonexistent@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(login_resp.status(), StatusCode::NOT_FOUND);

    common::cleanup(&db_file);
}

#[actix_rt::test]
async fn test_login_wrong_password() {
    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    // Register
    let register_resp = server.post("/api/v1/auth/register")
        .send_json(&json!({
            "email": "test5@example.com",
            "password_hash": "hash123"
        }))
        .await
        .unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    // Try login with wrong password
    let login_resp = server.post("/api/v1/auth/login")
        .send_json(&json!({
            "email": "test5@example.com",
            "password_hash": "wrong_hash"
        }))
        .await
        .unwrap();
    assert_eq!(login_resp.status(), StatusCode::UNAUTHORIZED);

    common::cleanup(&db_file);
}
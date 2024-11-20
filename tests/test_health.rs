mod common;

#[actix_rt::test]
async fn test_health_route() { 

    let (jwt_auth, db_file) = common::setup();
    let server = common::create_server(jwt_auth);

    let req = server.get("/api/v1/health").send();

    let response = req.await.unwrap();
    
    assert!(response.status().is_success());

    common::cleanup(&db_file);
}
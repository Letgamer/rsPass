use actix_web::{test, App, http::StatusCode};
use backend_rspass::{routes::route_health}; // Import the route handler from your project

#[actix_rt::test]
async fn test_health_route() {
    // Create a test app
    let app = test::init_service(App::new().service(route_health)).await;

    // Send a GET request to /api/v1/health
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert the response status is OK (200)
    assert_eq!(resp.status(), StatusCode::OK);
}
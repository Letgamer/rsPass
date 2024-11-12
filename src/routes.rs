use actix_web::{get, HttpResponse, Responder};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        route_health
    ),
    tags(
        (name = "health", description = "Health check endpoints")
    ),
    components(
        schemas()
    )
)]
pub struct ApiDoc;

/// Check the health status of the API
#[utoipa::path(
    responses(
        (status = 200, description = "API is healthy")
    ),
    tag = "health"
)]
#[get("/api/health")]
pub async fn route_health() -> impl Responder {
    HttpResponse::Ok().finish()
}
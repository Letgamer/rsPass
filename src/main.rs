use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use dotenv::dotenv;
use log::info;
use std::env;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_actix_web::AppExt;
use utoipa::OpenApi;

mod models;
mod routes;
use crate::routes::*;

fn get_server_config() -> (String, String) {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    (host, port)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    let (host, port) = get_server_config();
    info!("Starting server at {}:{}", host, port);

    HttpServer::new(|| {
        let (app, _api_doc) = App::new()
            .wrap(Logger::default())
            .into_utoipa_app()
            .service(route_health)
            .split_for_parts();

        app.service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
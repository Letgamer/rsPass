use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use env_logger::Env;
use log::info;
use std::{env, sync::Arc};
use tokio::{spawn, time::{self, Duration}};
use utoipa::{OpenApi};
use utoipa_actix_web::{AppExt, scope};
use utoipa_swagger_ui::SwaggerUi;

use backend_rspass::{
    auth::{JwtAuth, validator}, 
    db::initialize_database, 
    routes::*};

fn get_server_config() -> (String, String) {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    (host, port)
}

async fn run_blacklist_cleanup(jwt_auth: Arc<JwtAuth>) {
    let interval_seconds = env::var("CLEANUP_INTERVAL")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(600);

    info!("Cleanup Interval is set to: {} seconds", interval_seconds);

    let mut interval = time::interval(Duration::from_secs(interval_seconds));
    interval.tick().await;
    loop {
        interval.tick().await;
        info!("Running blacklist cleanup...");
        jwt_auth.cleanup_blacklist();
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    if let Err(e) = initialize_database() {
        panic!("Failed to initialize test database: {}", e);
    }

    let (host, port) = get_server_config();
    info!("Starting server at {}:{}", host, port);

    // Create JWT auth instance to share across workers
    let jwt_auth = Arc::new(JwtAuth::new());

    // Spawn cleanup task
    let cleanup_auth = jwt_auth.clone();
    spawn(async move { run_blacklist_cleanup(cleanup_auth).await });

    HttpServer::new(move|| {
        let auth = HttpAuthentication::with_fn(validator);
        let (app, _api_doc) = App::new()
            .wrap(Logger::default())
            .app_data(web::Data::from(jwt_auth.clone()))
            .into_utoipa_app()
            .service(route_health)
            .service(route_email)
            .service(route_login)
            .service(route_register)
            .service(
                scope("/api/v1/account")
                    .wrap(auth.clone())
                    .route("/changepwd", web::post().to(route_changepwd))
                    .route("/logout", web::get().to(route_logout))
                    .route("/delete", web::get().to(route_delete))
            )
            .service(
                scope("/api/v1/sync")
                    .wrap(auth)
                    .route("/fetch", web::get().to(route_fetch))
                    .route("/update", web::post().to(route_update))
            )
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
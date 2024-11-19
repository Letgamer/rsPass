use actix_web::web::Data;
use backend_rspass::{auth::JwtAuth, db::initialize_database, routes::*};
use std::{env, fs, sync::{Arc, Once}};
use actix_web::App;
use actix_test::TestServer;
use actix_web::web;
use actix_web::web::scope;
use backend_rspass::auth::validator;
use actix_web_httpauth::middleware::HttpAuthentication;

static INIT: Once = Once::new();

pub fn setup() -> Data<JwtAuth> {
    INIT.call_once(|| {
        env_logger::init();
        std::env::set_var("JWT_SECRET", "test_secret");
        std::env::set_var("DB_FILE", "./test.db");
        initialize_database();
    });

    // Initialize JwtAuth with test secret
    let jwt_auth = Arc::new(JwtAuth::new());

    jwt_auth.into()
}

pub fn cleanup() {
    if let Ok(db_file) = env::var("DB_FILE") {
        if fs::remove_file(db_file).is_err() {
            eprintln!("Failed to delete test.db");
        }
    }
}

pub fn create_server(jwt_auth: Data<JwtAuth>) -> TestServer {
    actix_test::start(move || {
        let auth = HttpAuthentication::with_fn(validator);
        App::new()
            .app_data(jwt_auth.clone())
            .service(route_health)
            .service(route_email)
            .service(route_login)
            .service(route_register)
            .service(
                scope("/api/v1/accounts")
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
    })
}
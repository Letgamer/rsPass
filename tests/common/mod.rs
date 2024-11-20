use actix_web::web::Data;
use backend_rspass::{auth::JwtAuth, db::initialize_database, routes::*};
use std::{env, fs, sync::Once};
use actix_web::App;
use actix_test::TestServer;
use actix_web::web;
use actix_web::web::scope;
use backend_rspass::auth::validator;
use actix_web_httpauth::middleware::HttpAuthentication;
use uuid::Uuid;

static INIT: Once = Once::new();

pub fn setup() -> (Data<JwtAuth>, String) {
    INIT.call_once(|| {
        env_logger::init();
    });

    let test_db = format!("./test_{}.db", Uuid::new_v4());
    env::set_var("DB_FILE", &test_db);
    if let Err(e) = initialize_database() {
        panic!("Failed to initialize test database: {}", e);
    }

    // Initialize JwtAuth with test secret
    std::env::set_var("JWT_SECRET", "test_secret");

    (Data::new(JwtAuth::new()), test_db)
}

pub fn cleanup(db_file: &str) {
    if let Ok(conn) = rusqlite::Connection::open(db_file) {
        let _ = conn.close();
    }
    if fs::remove_file(db_file).is_err() {
        println!("Failed to delete {}", db_file);
    }
    env::remove_var("DB_FILE");
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
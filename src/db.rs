use log::{error, info};
use rusqlite::Connection;
use std::path::Path;
use std::process;
use std::fs::OpenOptions;
use std::io::ErrorKind;

pub fn initialize_database(db_path: &str) {
    if !Path::new(db_path).exists() {
        info!("Creating new database at: {}", db_path);
        // Handle errors when creating the database
        if let Err(e) = Connection::open(db_path).and_then(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    email TEXT PRIMARY KEY,
                    password TEXT NOT NULL,
                    data TEXT NOT NULL
                );",
                [],
            )
        }) {
            error!("Failed to initialize database: {}", e);
            process::exit(1); // Exit the program with an error code
        }
        info!("Database created successfully.");
    } else {
        info!("Database already exists at: {}", db_path);

        // Check if the database is readable and writable
        match OpenOptions::new().read(true).write(true).open(db_path) {
            Ok(_) => {
                info!("Database is readable and writable.");
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => {
                        error!("Database file not found.");
                    }
                    ErrorKind::PermissionDenied => {
                        error!("No permission to read/write the database.");
                    }
                    _ => {
                        error!("Failed to access database: {}", e);
                    }
                }
                process::exit(1); // Exit with error if not readable and writable
            }
        }
    }
}

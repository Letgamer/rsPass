use log::{error, info};
use rusqlite::{params, Connection, Result};
use std::{env, fs::OpenOptions, io::ErrorKind, path::Path, process};

pub fn get_db_path() -> String {
    env::var("DB_FILE").unwrap_or_else(|_| "./database.db".to_string())
}

fn get_connection() -> Result<Connection> {
    let db_path = get_db_path();
    Connection::open(db_path)
}

pub fn initialize_database() -> Result<()> {
    let db_path = get_db_path();
    if !Path::new(&db_path).exists() {
        info!("Creating new database at: {}", db_path);
        if let Err(e) = get_connection().and_then(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    email TEXT PRIMARY KEY,
                    password_hash TEXT NOT NULL,
                    encrypted_data TEXT DEFAULT ''
                );",
                [],
            )
        }) {
            error!("Failed to initialize database: {}", e);
            process::exit(1);
        }
        info!("Database created successfully.");
    } else {
        if let Err(e) = OpenOptions::new().read(true).write(true).open(&db_path) {
            match e.kind() {
                ErrorKind::NotFound => error!("Database file not found."),
                ErrorKind::PermissionDenied => error!("No permission to read/write the database."),
                _ => error!("Failed to access database: {}", e),
            }
            process::exit(1);
        }
        info!("Database already existing at: {} is being used", db_path);
    }
    Ok(())
}

pub fn user_exists(email: &str) -> Result<bool> {
    let conn = get_connection()?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1)",
        params![email],
        |row| row.get(0),
    )?;
    Ok(exists)
}

pub fn user_login(email: &str, password_hash: &str) -> Result<bool> {
    let conn = get_connection()?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1 AND password_hash = ?2)",
        params![email, password_hash],
        |row| row.get(0),
    )?;
    Ok(exists)
}

pub fn user_register(email: &str, password_hash: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO users (email, password_hash) VALUES (?1, ?2)",
        params![email, password_hash],
    )?;
    Ok(())
}

pub fn user_changepwd(email: &str, password_hash: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE users SET password_hash = ?1 WHERE email = ?2",
        params![password_hash, email],
    )?;
    Ok(())
}

pub fn user_delete(email: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM users WHERE email = ?1", params![email])?;
    Ok(())
}

pub fn data_get(email: &str) -> Result<String> {
    let conn = get_connection()?;
    let encrypted_data: String = conn.query_row(
        "SELECT encrypted_data FROM users WHERE email = ?1",
        params![email],
        |row| row.get(0),
    )?;
    Ok(encrypted_data)
}

pub fn data_update(email: &str, encrypted_data: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE users SET encrypted_data = ?1 WHERE email = ?2",
        params![encrypted_data, email],
    )?;
    Ok(())
}

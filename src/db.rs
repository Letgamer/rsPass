use log::{error, info};
use rusqlite::{params, Connection, Result};
use std::{env, path::Path, process};

pub fn get_db_path() -> String {
    env::var("DB_FILE").unwrap_or_else(|_| "./database.db".to_string())
}

fn get_connection() -> Result<Connection> {
    let db_path = get_db_path();
    Connection::open(db_path)
}

pub fn initialize_database() -> Result<()> {
    let db_path = get_db_path();

    // Attempt to open the database
    match get_connection() {
        Ok(conn) => {
            info!("Database at {} opened successfully.", db_path);
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    email TEXT PRIMARY KEY,
                    password_hash TEXT NOT NULL,
                    encrypted_data TEXT DEFAULT ''
                );",
                [],
            )?;
            info!("Database initialized successfully.");
        }
        Err(e) => {
            if Path::new(&db_path).exists() {
                error!("Failed to open existing database: {}", e);
            } else {
                error!("Database file not found, and creation failed: {}", e);
            }
            process::exit(1);
        }
    }
    Ok(())
}

pub fn user_exists(email: &str) -> Result<bool> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1)",
        params![email],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(exists)
}

pub fn user_login(email: &str, password_hash: &str) -> Result<bool> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1 AND password_hash = ?2)", 
        params![email, password_hash],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(exists)
}

pub fn user_register(email: &str, password_hash: &str) -> Result<()> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO users (email, password_hash) VALUES (?1, ?2)",
        params![email, password_hash],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn user_changepwd(email: &str, password_hash: &str) -> Result<()> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE users SET password_hash = ?1 WHERE email = ?2",
        params![password_hash, email],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn user_delete(email: &str) -> Result<()> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM users WHERE email = ?1", 
        params![email]
    )?;
    tx.commit()?;
    Ok(())
}

pub fn data_get(email: &str) -> Result<String> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    let encrypted_data: String = tx.query_row(
        "SELECT encrypted_data FROM users WHERE email = ?1",
        params![email],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(encrypted_data)
}

pub fn data_update(email: &str, encrypted_data: &str) -> Result<()> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE users SET encrypted_data = ?1 WHERE email = ?2",
        params![encrypted_data, email],
    )?;
    tx.commit()?;  
    Ok(())
}

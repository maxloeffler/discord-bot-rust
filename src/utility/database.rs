
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

use std::sync::Arc;
use std::collections::HashSet;
use std::fmt;

use crate::utility::traits::ToList;


#[derive(EnumIter, Clone)]
pub enum DB {
    Config,
    Warnings,
    Mutes,
    Flag,
    Bans,
}

impl fmt::Display for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DB::Config => write!(f, "config"),
            DB::Warnings => write!(f, "warnings"),
            DB::Mutes => write!(f, "mutes"),
            DB::Flag => write!(f, "flag"),
            DB::Bans => write!(f, "bans"),
        }
    }
}

pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {

    pub fn new() -> Self {
        let path = "src/database.db".to_string();
        let connection = Connection::open(path).expect("Failed to open database");
        Database { connection: Arc::new(Mutex::new(connection)) }
    }

    pub async fn init(&self) {
        let connection = self.connection.lock().await;
        for db in DB::iter() {
            connection.execute(&format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id      INTEGER PRIMARY KEY,
                    key     TEXT    NOT NULL,
                    value   TEXT    NOT NULL
                )", db.to_string()),
                [],
            ).expect("Failed to create table");
        }
    }

    pub async fn get_keys(&self, db: DB) -> Vec<String> {
        let connection = self.connection.lock().await;
        let mut keys = HashSet::new();
        let mut statement = connection.prepare(
            &format!("SELECT key FROM {}", db.to_string())
        ).expect("Failed to prepare statement");
        let rows = statement.query_map(
            [],
            |row| row.get(0)
        ).expect("Failed to query map");
        for key in rows {
            keys.insert(key.unwrap());
        }
        keys.into_iter().collect()
    }

    pub async fn get_multiple(&self, db: DB, keys: impl ToList<&str>) -> Option<Vec<String>> {
        let connection = self.connection.lock().await;
        let mut values = Vec::new();
        for key in keys.to_list() {
            let value = connection.query_row(
                &format!("SELECT value FROM {} WHERE key = ?", db.to_string()),
                params![key],
                |row| row.get(0),
            );
            match value {
                Ok(value) => values.push(value),
                Err(_) => return None,
            };
        }
        Some(values)
    }

    pub async fn get(&self, db: DB, key: &str) -> Option<String> {
        let connection = self.connection.lock().await;
        let value = connection.query_row(
            &format!("SELECT value FROM {} WHERE key = ?", db.to_string()),
            params![key.to_string()],
            |row| row.get(0),
        );
        match value {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub async fn set(&self, db: DB, key: &str, value: impl ToList<&str>) {

        let connection = self.connection.lock().await;
        for value in value.to_list() {

            // Delete old values
            connection.execute(
                &format!("DELETE FROM {} WHERE key = ?", db.to_string()),
                params![key],
            ).expect("Failed to delete value");

            connection.execute(
                &format!("INSERT INTO {} (key, value) VALUES (?, ?)", db.to_string()),
                params![key, value],
            ).expect("Failed to set value");

        }
    }

    pub async fn append(&self, db: DB, key: &str, value: &str) {
        let connection = self.connection.lock().await;
        connection.execute(
            &format!("INSERT INTO {} (key, value) VALUES (?, ?)", db.to_string()),
            params![key, value],
        ).expect("Failed to append value");
    }

    pub async fn delete(&self, db: DB, key: &str) {
        let connection = self.connection.lock().await;
        connection.execute(
            &format!("DELETE FROM {} WHERE key = ?", db.to_string()),
            params![key],
        ).expect("Failed to delete value");
    }

}

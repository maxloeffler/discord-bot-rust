
use rusqlite::{params, Connection};
use tokio::sync::Mutex;

use std::sync::Arc;
use std::collections::HashSet;

use crate::utility::mixed_utility::ToList;


pub struct Database {
    name: String,
    connection: Arc<Mutex<Connection>>,
}

impl Database {

    pub fn new(name: &str) -> Self {
        let connection = Connection::open(format!("src/databases/{}.db", name)).expect("Failed to open database");
        connection.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id      INTEGER PRIMARY KEY,
                key     TEXT    NOT NULL,
                value   TEXT    NOT NULL
            )", name),
            [],
        ).expect("Failed to create table");
        Database { name: name.to_string(), connection: Arc::new(Mutex::new(connection)) }
    }

    pub async fn get_keys(&self) -> Vec<String> {
        let connection = self.connection.lock().await;
        let mut keys = HashSet::new();
        let mut statement = connection.prepare(
            &format!("SELECT key FROM {}", self.name)
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

    pub async fn get_multiple(&self, keys: impl ToList<&str>) -> Option<Vec<String>> {
        let connection = self.connection.lock().await;
        let mut values = Vec::new();
        for key in keys.to_list() {
            let value = connection.query_row(
                &format!("SELECT value FROM {} WHERE key = ?", self.name),
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

    pub async fn get(&self, key: &str) -> Option<String> {
        let connection = self.connection.lock().await;
        let value = connection.query_row(
            &format!("SELECT value FROM {} WHERE key = ?", self.name),
            params![key.to_string()],
            |row| row.get(0),
        );
        match value {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub async fn set(&self, key: &str, value: impl ToList<&str>) {

        let connection = self.connection.lock().await;
        for value in value.to_list() {

            // Delete old values
            connection.execute(
                &format!("DELETE FROM {} WHERE key = ?", self.name),
                params![key],
            ).expect("Failed to delete value");

            connection.execute(
                &format!("INSERT INTO {} (key, value) VALUES (?, ?)", self.name),
                params![key, value],
            ).expect("Failed to set value");

        }
    }

    pub async fn delete(&self, key: &str) {
        let connection = self.connection.lock().await;
        connection.execute(
            &format!("DELETE FROM {} WHERE key = ?", self.name),
            params![key],
        ).expect("Failed to delete value");
    }

}

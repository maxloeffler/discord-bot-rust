
use rusqlite::{params, Connection};
use tokio::sync::Mutex;

use std::sync::Arc;
use std::collections::HashSet;


trait DatabaseArg {
    fn to_list(&self) -> Vec<String>;
}
impl DatabaseArg for Vec<String> {
    fn to_list(&self) -> Vec<String> {
        self.clone()
    }
}
impl DatabaseArg for String {
    fn to_list(&self) -> Vec<String> {
        vec![self.clone()]
    }
}
impl DatabaseArg for &str {
    fn to_list(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}
impl DatabaseArg for str {
    fn to_list(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}
impl DatabaseArg for [&str] {
    fn to_list(&self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}


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

    pub async fn get_multiple<T: DatabaseArg + ?Sized>(&self, keys: &T) -> Vec<String> {
        let connection = self.connection.lock().await;
        let mut values = Vec::new();
        for key in keys.to_list() {
            let value: String = connection.query_row(
                &format!("SELECT value FROM {} WHERE key = ?", self.name),
                params![key],
                |row| row.get(0),
            ).expect("Failed to get value");
            values.push(value);
        }
        values
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

    pub async fn set<T: DatabaseArg + ?Sized>(&self, key: &str, value: &T) {

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

}

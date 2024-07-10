
use rusqlite::{params, Connection};
use tokio::sync::Mutex;

use std::sync::Arc;


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


pub struct Database {
    name: String,
    connection: Arc<Mutex<Connection>>,
}

impl Database {

    pub fn new(name: &str) -> Self {
        let connection = Connection::open(format!("../databases/{}.db", name)).expect("Failed to open database");
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

    pub async fn get_multiple<T: DatabaseArg>(&self, keys: T) -> Vec<String> {
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

    pub async fn get(&self, key: &str) -> String {
        let connection = self.connection.lock().await;
        let value: String = connection.query_row(
            &format!("SELECT value FROM {} WHERE key = ?", self.name),
            params![key],
            |row| row.get(0),
        ).expect("Failed to get value");
        value
    }

    pub async fn set<T: DatabaseArg>(&self, key: T, value: T) {

        if key.to_list().len() != value.to_list().len() {
            panic!("Key and value lengths do not match");
        }

        let connection = self.connection.lock().await;
        for (key, value) in key.to_list().iter().zip(value.to_list().iter()) {

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

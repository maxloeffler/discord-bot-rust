
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use strum_macros::EnumIter;

use std::sync::Arc;
use std::collections::HashSet;
use std::fmt;

use crate::utility::traits::ToList;
use crate::utility::mixed::{BoxedFuture, Result};


#[derive(EnumIter, Clone)]
pub enum DB {
    Config,
    Warnings,
    Mutes,
    Flags,
    Bans,
}

impl fmt::Display for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DB::Config => write!(f, "config"),
            DB::Warnings => write!(f, "warnings"),
            DB::Mutes => write!(f, "mutes"),
            DB::Flags => write!(f, "flags"),
            DB::Bans => write!(f, "bans"),
        }
    }
}

#[derive(Debug)]
pub struct DBEntry {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub timestamp: i64,
}

impl Into<String> for DBEntry {
    fn into(self) -> String {
        self.value.clone()
    }
}

impl fmt::Display for DBEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    pub identifier: DB,
}

impl Database {

    pub fn new(identifier: DB) -> Self {
        let path = format!("src/databases/{}.db", identifier.to_string());
        let connection = Connection::open(path).expect("Failed to open database");
        connection.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id        INTEGER PRIMARY KEY,
                key       TEXT    NOT NULL,
                value     TEXT    NOT NULL,
                timestamp INTEGER
            )", identifier.to_string()),
            [],
        ).expect("Failed to create table");
        Database { connection: Arc::new(Mutex::new(connection)), identifier: identifier }
    }

    pub async fn get_keys(&self) -> Vec<String> {
        let connection = self.connection.lock().await;
        let mut keys = HashSet::new();
        let mut statement = connection.prepare(
            &format!("SELECT key FROM {}", self.identifier.to_string())
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

    pub async fn query(&self, key: &str, query_string: &str) -> Result<Vec<DBEntry>> {
        let connection = self.connection.lock().await;
        let mut statement = connection.prepare(&format!(
            "SELECT id, value, timestamp FROM {} WHERE key = ? {}",
            self.identifier.to_string(),
            query_string
        )).expect("Failed to prepare statement");
        let entry_iter = statement.query_map([key], |entry| {
            Ok(DBEntry {
                id: entry.get(0)?,
                key: key.to_string(),
                value: entry.get(1)?,
                timestamp: entry.get(2)?,
            })
        }).expect("Failed to query map");
        Ok(entry_iter.map(|entry| entry.unwrap()).collect::<Vec<DBEntry>>())
    }

    pub async fn get(&self, key: &str) -> Result<DBEntry> {
        self.query(key, "ORDER BY timestamp ASC LIMIT 1").await
            .map(|mut entries|
                entries.pop().ok_or(&format!("Failed to get value for '{}'", key)).unwrap())
    }

    pub async fn get_all(&self, key: &str) -> Result<Vec<DBEntry>> {
        self.query(key, "").await
    }

    pub async fn get_last(&self, key: &str, limit: u8) -> Result<Vec<DBEntry>> {
        self.query(key, &format!("ORDER BY timestamp DESC LIMIT {}", limit)).await
    }

    pub async fn get_multiple(&self, keys: impl ToList<&str>) -> Result<Vec<DBEntry>> {
        let mut values = Vec::new();
        for key in keys.to_list() {
            let value = self.get(key).await?;
            values.push(value);
        }
        Ok(values)
    }

    pub async fn set(&self, key: &str, value: impl ToList<&str>) {

        let connection = self.connection.lock().await;
        for value in value.to_list() {

            // Delete old values
            connection.execute(
                &format!("DELETE FROM {} WHERE key = ?", self.identifier.to_string()),
                params![key],
            ).expect("Failed to delete value");

            connection.execute(
                &format!("INSERT INTO {} (key, value, timestamp) VALUES (?, ?, ?)", self.identifier.to_string()),
                params![key, value, chrono::Utc::now().timestamp_millis()],
            ).expect("Failed to set value");

        }
    }

    pub async fn append(&self, key: &str, value: &str) {
        let connection = self.connection.lock().await;
        connection.execute(
            &format!("INSERT INTO {} (key, value, timestamp) VALUES (?, ?, ?)", self.identifier.to_string()),
            params![key, value, chrono::Utc::now().timestamp_millis()],
        ).expect("Failed to append value");
    }

    pub async fn delete(&self, key: &str) {
        let connection = self.connection.lock().await;
        connection.execute(
            &format!("DELETE FROM {} WHERE key = ?", self.identifier.to_string()),
            params![key],
        ).expect("Failed to delete value");
    }

}


// ---- Wrapper Classes for improved Concurrency ---- //

pub trait DatabaseWrapper {

    fn get_database(&self) -> Database;

    fn get_keys<'a>(&'a self) -> BoxedFuture<'a, Vec<String>>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().get_keys().await
        })
    }

    fn get<'a>(&'a self, key: &'a str) -> BoxedFuture<'a, Result<DBEntry>>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().get(key).await
        })
    }

    fn query<'a>(&'a self, key: &'a str, query_string: &'a str) -> BoxedFuture<'a, Result<Vec<DBEntry>>>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().query(key, query_string).await
        })
    }


    fn get_all<'a>(&'a self, key: &'a str) -> BoxedFuture<'a, Result<Vec<DBEntry>>>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().get_all(key).await
        })
    }

    fn get_last<'a>(&'a self, key: &'a str, limit: u8) -> BoxedFuture<'a, Result<Vec<DBEntry>>>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().get_last(key, limit).await
        })
    }

    fn get_multiple<'a>(&'a self, keys: Vec<&'a str>) -> BoxedFuture<'a, Result<Vec<DBEntry>>>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().get_multiple(keys).await
        })
    }

    fn set<'a>(&'a self, key: &'a str, value: &'a str) -> BoxedFuture<'a, ()>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().set(key, vec![value]).await
        })
    }

    fn append<'a>(&'a self, key: &'a str, value: &'a str) -> BoxedFuture<'a, ()>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().append(key, value).await
        })
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxedFuture<'a, ()>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().delete(key).await
        })
    }
}

pub struct ConfigDB { database: Database }

impl ConfigDB {
    pub fn new() -> Self {
        ConfigDB { database: Database::new(DB::Config) }
    }
}

impl DatabaseWrapper for ConfigDB {
    fn get_database(&self) -> Database {
        self.database.clone()
    }
}

pub struct WarningsDB { database: Database }

impl WarningsDB {
    pub fn new() -> Self {
        WarningsDB { database: Database::new(DB::Warnings) }
    }
}

impl DatabaseWrapper for WarningsDB {
    fn get_database(&self) -> Database {
        self.database.clone()
    }
}

pub struct MutesDB { database: Database }

impl MutesDB {
    pub fn new() -> Self {
        MutesDB { database: Database::new(DB::Mutes) }
    }
}

impl DatabaseWrapper for MutesDB {
    fn get_database(&self) -> Database {
        self.database.clone()
    }
}

pub struct FlagsDB { database: Database }

impl FlagsDB {
    pub fn new() -> Self {
        FlagsDB { database: Database::new(DB::Flags) }
    }
}

impl DatabaseWrapper for FlagsDB {
    fn get_database(&self) -> Database {
        self.database.clone()
    }
}

pub struct BansDB { database: Database }

impl BansDB {
    pub fn new() -> Self {
        BansDB { database: Database::new(DB::Bans) }
    }
}

impl DatabaseWrapper for BansDB {
    fn get_database(&self) -> Database {
        self.database.clone()
    }
}


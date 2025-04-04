
use serenity::all::{ChannelId, UserId};
use serenity::model::colour::Colour;
use rusqlite::{params, Connection};
use strum_macros::EnumIter;

use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashSet;
use std::fmt;
use std::sync::RwLock;

use crate::utility::*;


#[derive(EnumIter, Clone, Hash, PartialEq, Eq, Debug)]
pub enum DB {
    Config,
    Warnings,
    Mutes,
    Unmutes,
    Flags,
    Bans,
    Afk,
    Schedule,
    TicketReviews,
    Notes,
    Reminders,
    Tweets,
    Deadchat
}

impl fmt::Display for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DB::Config => write!(f, "config"),
            DB::Warnings => write!(f, "warnings"),
            DB::Mutes => write!(f, "mutes"),
            DB::Unmutes => write!(f, "unmutes"),
            DB::Flags => write!(f, "flags"),
            DB::Bans => write!(f, "bans"),
            DB::Afk => write!(f, "afk"),
            DB::Schedule => write!(f, "schedule"),
            DB::TicketReviews => write!(f, "ticket_reviews"),
            DB::Notes => write!(f, "notes"),
            DB::Reminders => write!(f, "reminders"),
            DB::Tweets => write!(f, "tweets"),
            DB::Deadchat => write!(f, "deadchat"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl From<DBEntry> for Colour {
    fn from(entry: DBEntry) -> Colour {
        let value = entry.value.trim_start_matches("#");
        u64::from_str_radix(value, 16).unwrap().into()
    }
}

impl From<DBEntry> for ChannelId {
    fn from(entry: DBEntry) -> ChannelId {
        ChannelId::from_str(&entry.value).unwrap()
    }
}

impl From<DBEntry> for UserId {
    fn from(entry: DBEntry) -> UserId {
        UserId::from_str(&entry.value).unwrap()
    }
}

pub struct Database {
    connection: RwLock<Connection>,
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
        Database { connection: RwLock::new(connection), identifier: identifier }
    }

    pub async fn get_keys(&self) -> Vec<String> {
        let connection = self.connection.read().expect("Failed to get connection");
        let mut keys = HashSet::new();
        let mut statement = connection
            .prepare(&format!("SELECT key FROM {}", self.identifier.to_string()))
            .expect("Failed to prepare statement");
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
        let connection = self.connection.read().expect("Failed to get connection");
        let mut statement = connection.prepare(&format!(
            "SELECT id, key, value, timestamp FROM {} WHERE key = ? {}",
            self.identifier.to_string(),
            query_string
        )).expect("Failed to prepare statement");
        let entry_iter = statement.query_map([key], |entry| {
            Ok(DBEntry {
                id: entry.get(0)?,
                key: entry.get(1)?,
                value: entry.get(2)?,
                timestamp: entry.get(3)?,
            })
        }).expect("Failed to query map");
        Ok(entry_iter.map(|entry| entry.unwrap()).collect::<Vec<DBEntry>>())
    }

    pub async fn get(&self, key: &str) -> Result<DBEntry> {
        self.query(key, "ORDER BY timestamp ASC LIMIT 1").await?
            .pop()
            .ok_or(format!("Failed to get value for '{}'", key))
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

        let connection = self.connection.write().expect("Failed to get connection");
        for value in value.to_list() {

            // Delete old values
            connection.execute(
                &format!("DELETE FROM {} WHERE key = ?", self.identifier.to_string()),
                params![key],
            ).expect("Failed to delete value");

            connection.execute(
                &format!("INSERT INTO {} (key, value, timestamp) VALUES (?, ?, ?)", self.identifier.to_string()),
                params![key, value, chrono::Utc::now().timestamp()],
            ).expect("Failed to set value");

        }
    }

    pub async fn has(&self, key: &str) -> bool {

        let connection = self.connection.read().expect("Failed to get connection");
        let mut statement = connection.prepare(&format!(
            "SELECT id FROM {} WHERE key = ?",
            self.identifier.to_string()
        )).expect("Failed to prepare statement");

        let entry_iter = statement.query_map([key], |_| Ok(())).expect("Failed to query map");
        entry_iter.count() > 0
    }

    pub async fn append(&self, key: &str, value: &str) {
        let connection = self.connection.write().expect("Failed to get connection");
        connection.execute(
            &format!("INSERT INTO {} (key, value, timestamp) VALUES (?, ?, ?)", self.identifier.to_string()),
            params![key, value, chrono::Utc::now().timestamp()],
        ).expect("Failed to append value");
    }

    pub async fn delete(&self, key: &str) {
        let connection = self.connection.write().expect("Failed to get connection");
        connection.execute(
            &format!("DELETE FROM {} WHERE key = ?", self.identifier.to_string()),
            params![key],
        ).expect("Failed to delete value");
    }

    pub async fn delete_by_id(&self, id: i64) {
        let connection = self.connection.write().expect("Failed to get connection");
        connection.execute(
            &format!("DELETE FROM {} WHERE id = ?", self.identifier.to_string()),
            params![id],
        ).expect("Failed to delete value");
    }
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

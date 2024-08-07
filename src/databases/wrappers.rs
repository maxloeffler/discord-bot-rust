
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use std::sync::Arc;

use crate::databases::database::Database;
use crate::databases::database::DBEntry;
use crate::databases::database::DB;
use crate::utility::*;
use crate::impl_singleton;


#[derive(Serialize, Deserialize)]
pub struct ModLog {
    pub staff_id: String,
    pub member_id: String,
    pub reason: String,
}

impl ModLog {
    pub fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl From<&DBEntry> for ModLog {
    fn from(entry: &DBEntry) -> Self {
        serde_json::from_str(&entry.value).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FlagLog {
    pub staff_id: String,
    pub member_id: String,
    pub reason: String,
    pub monthly: bool,
}

impl FlagLog {
    pub fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    pub fn is_active(&self, issuance_date: i64) -> bool {
        let duration = match self.monthly {
            true  => 30 * 24 * 60 * 60,
            false =>  7 * 24 * 60 * 60
        };
        let now = chrono::Utc::now().timestamp();
        let expiration_date = issuance_date + duration;
        expiration_date > now
    }
}

impl From<&DBEntry> for FlagLog {
    fn from(entry: &DBEntry) -> Self {
        serde_json::from_str(&entry.value).unwrap()
    }
}

pub trait DatabaseWrapper: Send + Sync {

    fn get_database(&self) -> &Database;

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

    fn delete_by_id<'a>(&'a self, id: i64) -> BoxedFuture<'a, ()>
        where Self: Sync
    {
        Box::pin(async move {
            self.get_database().delete_by_id(id).await
        })
    }
}

macro_rules! impl_database_wrapper {
    ($name:ident, $db_type:expr) => {
        pub struct $name {
            database: Database
        }

        impl $name {
            pub fn new() -> Self {
                $name { database: Database::new($db_type) }
            }
        }

        impl DatabaseWrapper for $name {
            fn get_database(&self) -> &Database {
                &self.database
            }
        }

        impl_singleton!($name);
    };
}

impl_database_wrapper!(ConfigDB, DB::Config);
impl_database_wrapper!(WarningsDB, DB::Warnings);
impl_database_wrapper!(MutesDB, DB::Mutes);
impl_database_wrapper!(BansDB, DB::Bans);
impl_database_wrapper!(FlagsDB, DB::Flags);
impl_database_wrapper!(AfkDB, DB::Afk);
impl_database_wrapper!(ScheduleDB, DB::Schedule);

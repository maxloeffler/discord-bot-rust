
use serde::{Serialize, Deserialize};

use crate::databases::database::Database;
use crate::databases::database::DBEntry;
use crate::databases::database::DB;
use crate::utility::*;



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

pub trait DatabaseWrapper {

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
}

pub struct ConfigDB { database: Database }

impl ConfigDB {
    pub fn new() -> Self {
        ConfigDB { database: Database::new(DB::Config) }
    }
}

impl DatabaseWrapper for ConfigDB {
    fn get_database(&self) -> &Database {
        &self.database
    }
}

pub struct WarningsDB { database: Database }

impl WarningsDB {
    pub fn new() -> Self {
        WarningsDB { database: Database::new(DB::Warnings) }
    }
}

impl DatabaseWrapper for WarningsDB {
    fn get_database(&self) -> &Database {
        &self.database
    }
}

pub struct MutesDB { database: Database }

impl MutesDB {
    pub fn new() -> Self {
        MutesDB { database: Database::new(DB::Mutes) }
    }
}

impl DatabaseWrapper for MutesDB {
    fn get_database(&self) -> &Database {
        &self.database
    }
}

pub struct FlagsDB { database: Database }

impl FlagsDB {
    pub fn new() -> Self {
        FlagsDB { database: Database::new(DB::Flags) }
    }
}

impl DatabaseWrapper for FlagsDB {
    fn get_database(&self) -> &Database {
        &self.database
    }
}

pub struct BansDB { database: Database }

impl BansDB {
    pub fn new() -> Self {
        BansDB { database: Database::new(DB::Bans) }
    }
}

impl DatabaseWrapper for BansDB {
    fn get_database(&self) -> &Database {
        &self.database
    }
}

pub struct AfkDB { database: Database }

impl AfkDB {
    pub fn new() -> Self {
        AfkDB { database: Database::new(DB::Afk) }
    }
}

impl DatabaseWrapper for AfkDB {
    fn get_database(&self) -> &Database {
        &self.database
    }
}


use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use nonempty::{NonEmpty, nonempty};

use std::sync::Arc;
use std::convert::From;

use crate::databases::database::Database;
use crate::databases::database::DBEntry;
use crate::databases::database::DB;
use crate::utility::*;
use crate::impl_singleton;


macro_rules! as_db_entry {
    ($name:ident, $($field_name:ident: $field_type:ty),*) => {

        #[derive(Serialize, Deserialize, Clone)]
        pub struct $name {
            pub id: i64,
            pub key: String,
            pub timestamp: i64,
            $(pub $field_name: $field_type),*
        }

        impl $name {
            pub fn new($($field_name: $field_type),*) -> Self {
                $name {
                    id: 0,
                    key: "".to_string(),
                    timestamp: 0,
                    $($field_name),*
                }
            }
            pub fn into(self) -> String {
                let mut relevant_fields = Vec::<String>::new();
                for field in vec![$(self.$field_name.to_string()),*] {
                    relevant_fields.push(field);
                }
                serde_json::to_string(&relevant_fields).unwrap()
            }
        }

        impl From<DBEntry> for $name {
            fn from(entry: DBEntry) -> $name {
                let mut relevant: Vec<String> = serde_json::from_str(&entry.value).unwrap();
                relevant.reverse();
                $name {
                    id: entry.id,
                    key: entry.key,
                    timestamp: entry.timestamp,
                    $($field_name: relevant.pop().unwrap().parse().unwrap()),*
                }
            }
        }
    }
}

as_db_entry!(ModLog, staff_id: String, reason: String);
as_db_entry!(FlagLog, staff_id: String, reason: String, monthly: bool);
as_db_entry!(TicketReviewLog, reviewer_id: String, approved: bool, notes: String);

impl FlagLog {
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
impl From<FlagLog> for ModLog {
    fn from(flag_log: FlagLog) -> ModLog {
        ModLog {
            id: flag_log.id,
            key: flag_log.key,
            timestamp: flag_log.timestamp,
            staff_id: flag_log.staff_id,
            reason: flag_log.reason
        }
    }
}

as_db_entry!(ScheduleLog, expiration_date: i64, message: String, channel_id: String);

impl ScheduleLog {
    pub fn is_expired(&self, now: i64) -> bool {
        self.expiration_date < now
    }
}

as_db_entry!(Note, content: String);

impl Note {
    pub fn escape(key: String) -> String {
        key.replace(" ", "_")
    }
    pub fn deescape(key: String) -> String {
        key.replace("_", " ")
    }
}

impl Triggerable for Note {
    fn get_triggers(&self) -> NonEmpty<String> {
        nonempty![Note::deescape(self.key.clone())]
    }
}

pub trait DatabaseWrapper<T: From<DBEntry>>: Send + Sync {

    fn get_database(&self) -> &Database;

    fn get_keys<'a>(&'a self) -> BoxedFuture<'a, Vec<String>> {
        Box::pin(async move {
            self.get_database().get_keys().await
        })
    }

    fn get<'a>(&'a self, key: &'a str) -> BoxedFuture<'a, Result<T>> {
        Box::pin(async move {
            let entry = self.get_database().get(key).await;
            match entry {
                Ok(entry) => Ok(T::from(entry)),
                Err(_)    => Err("Key not found".to_string())
            }
        })
    }

    fn query<'a>(&'a self, key: &'a str, query_string: &'a str) -> BoxedFuture<'a, Result<Vec<T>>> {
        Box::pin(async move {
            let entries = self.get_database().query(key, query_string).await;
            match entries {
                Ok(entries) => Ok(entries.into_iter().map(|entry| T::from(entry)).collect()),
                Err(_)      => Err("Query failed".to_string())
            }
        })
    }

    fn get_all<'a>(&'a self, key: &'a str) -> BoxedFuture<'a, Result<Vec<T>>> {
        Box::pin(async move {
            let entries = self.get_database().get_all(key).await;
            match entries {
                Ok(entries) => Ok(entries.into_iter().map(|entry| T::from(entry)).collect()),
                Err(_)      => Err("Key not found".to_string())
            }
        })
    }

    fn get_last<'a>(&'a self, key: &'a str, limit: u8) -> BoxedFuture<'a, Result<Vec<T>>> {
        Box::pin(async move {
            let entries = self.get_database().get_last(key, limit).await;
            match entries {
                Ok(entries) => Ok(entries.into_iter().map(|entry| T::from(entry)).collect()),
                Err(_)      => Err("Key not found".to_string())
            }
        })
    }

    fn get_multiple<'a>(&'a self, keys: Vec<&'a str>) -> BoxedFuture<'a, Result<Vec<T>>> {
        Box::pin(async move {
            let entries = self.get_database().get_multiple(keys).await;
            match entries {
                Ok(entries) => Ok(entries.into_iter().map(|entry| T::from(entry)).collect()),
                Err(_)      => Err("Key not found".to_string())
            }
        })
    }

    fn set<'a>(&'a self, key: &'a str, value: &'a str) -> BoxedFuture<'a, ()> {
        Box::pin(async move {
            self.get_database().set(key, vec![value]).await
        })
    }

    fn append<'a>(&'a self, key: &'a str, value: &'a str) -> BoxedFuture<'a, ()> {
        Box::pin(async move {
            self.get_database().append(key, value).await
        })
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxedFuture<'a, ()> {
        Box::pin(async move {
            self.get_database().delete(key).await
        })
    }

    fn delete_by_id<'a>(&'a self, id: i64) -> BoxedFuture<'a, ()> {
        Box::pin(async move {
            self.get_database().delete_by_id(id).await
        })
    }
}

macro_rules! impl_database_wrapper {

    ($name:ident, $db_type:expr, ModLog) => {
        pub struct $name {
            database: Database
        }

        impl DatabaseWrapper<ModLog> for $name {
            fn get_database(&self) -> &Database {
                &self.database
            }
        }

        impl $name {
            pub fn new() -> Self {
                $name { database: Database::new($db_type) }
            }

            // this function is an optional convenience function
            // but does not need to be called necessarily
            #[allow(unused)]
            pub fn get_by_staff<'a>(&'a self, staff_id: &'a str) -> BoxedFuture<'a, Vec<ModLog>> {
                Box::pin(async move {
                    self.query("", &format!("AND value LIKE '%staff_id%{}%'", staff_id)).await.unwrap()
                })
            }
        }

        impl_singleton!($name);
    };

    ($name:ident, $db_type:expr, $log_type:ty) => {
        pub struct $name {
            database: Database
        }

        impl $name {
            pub fn new() -> Self {
                $name { database: Database::new($db_type) }
            }
        }

        impl DatabaseWrapper<$log_type> for $name {
            fn get_database(&self) -> &Database {
                &self.database
            }
        }

        impl_singleton!($name);
    };

    ($name:ident, $db_type:expr) => {
        impl_database_wrapper!($name, $db_type, DBEntry);
    };
}

impl_database_wrapper!(ConfigDB, DB::Config);
impl_database_wrapper!(WarningsDB, DB::Warnings, ModLog);
impl_database_wrapper!(MutesDB, DB::Mutes, ModLog);
impl_database_wrapper!(UnmutesDB, DB::Unmutes, ModLog);
impl_database_wrapper!(BansDB, DB::Bans, ModLog);
impl_database_wrapper!(FlagsDB, DB::Flags, FlagLog);
impl_database_wrapper!(AfkDB, DB::Afk);
impl_database_wrapper!(ScheduleDB, DB::Schedule, ScheduleLog);
impl_database_wrapper!(RemindersDB, DB::Reminders, ScheduleLog);
impl_database_wrapper!(TicketReviewsDB, DB::TicketReviews, TicketReviewLog);
impl_database_wrapper!(NotesDB, DB::Notes, Note);
impl_database_wrapper!(TweetsDB, DB::Tweets);
impl_database_wrapper!(DeadchatDB, DB::Deadchat);

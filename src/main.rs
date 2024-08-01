
#![allow(unused_imports)]
use strum::IntoEnumIterator;
use tokio::runtime::Runtime;
use serenity::cache::Settings;
use serenity::prelude::{Client, GatewayIntents};
use tokio::sync::Mutex;

use std::sync::Arc;
use std::thread;

use commands::command_manager::CommandManager;
use handler::Handler;
use databases::*;
use utility::*;

mod handler;
mod utility;
mod databases;
mod commands;


#[tokio::main]
async fn main() {

    // setup
    let token = setup().await;
    let command_handler = CommandManager::new();
    let handler = Handler::new(command_handler);

    // configure cache
    let mut cache_settings = Settings::default();
    cache_settings.max_messages = 50;

    // start threads
    let intents = GatewayIntents::GUILDS                    |
                  GatewayIntents::GUILD_MESSAGES            |
                  GatewayIntents::GUILD_MEMBERS             |
                  GatewayIntents::GUILD_VOICE_STATES        |
                  GatewayIntents::GUILD_MESSAGE_REACTIONS   |
                  GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .cache_settings(cache_settings)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // start threads
    #[cfg(feature = "db_interface")]
    spawn_database_thread().await;

    let _ = client.start().await;
}


async fn setup() -> String {
    let config = ConfigDB::get_instance().lock().await;
    config.set("uptime", &chrono::Utc::now().timestamp().to_string()).await;
    config.set("token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    config.get("token").await.unwrap().to_string()
}

fn get_db(db: &DB) -> Arc<&'static Mutex<dyn DatabaseWrapper>> {
    match db {
        DB::Config => Arc::new(ConfigDB::get_instance() as &Mutex<dyn DatabaseWrapper>),
        DB::Mutes => Arc::new(MutesDB::get_instance() as &Mutex<dyn DatabaseWrapper>),
        DB::Warnings => Arc::new(WarningsDB::get_instance() as &Mutex<dyn DatabaseWrapper>),
        DB::Flags => Arc::new(FlagsDB::get_instance() as &Mutex<dyn DatabaseWrapper>),
        DB::Bans => Arc::new(BansDB::get_instance() as &Mutex<dyn DatabaseWrapper>),
        DB::Afk => Arc::new(AfkDB::get_instance() as &Mutex<dyn DatabaseWrapper>),
    }
}

#[cfg(feature = "db_interface")]
async fn spawn_database_thread() {
    let mut db = DB::Config;
    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        Logger::info_long("Connected to database", db.to_string().as_str());
        runtime.block_on(async {
            loop {
                let database = get_db(&db);
                let input = Logger::input("Enter a command");
                let words = input.split_whitespace().collect::<Vec<&str>>();

                match words[0] {
                    "ls" => {
                        let mut keys = database.lock().await.get_keys().await;
                        keys.sort();
                        Logger::info_long("Keys", &keys.join(", "));
                    }
                    "get" => {
                        match words.len() {
                            1 => {
                                Logger::warn("Too few parameters");
                            },
                            2 => {
                                let key = words[1];
                                let value = database.lock().await.get(key).await;
                                match value {
                                    Ok(value) => Logger::info_long(&format!("Value of {}", key), &value.to_string()),
                                    Err(err) => Logger::err(err.as_str())
                                }
                            }
                            _ => {
                                match words[1] {
                                    "all" => {
                                        let values = database.lock().await.get_all(words[2]).await;
                                        match values {
                                            Ok(values) => {
                                                let values: Vec<_> = values.iter().map(|entry| entry.to_string()).collect();
                                                Logger::info_long(&format!("Values of {}", words[2]), &values.join(", "))
                                            }
                                            Err(err) => Logger::err(err.as_str())
                                        }
                                    },
                                    _ => {
                                        let values = database.lock().await.get_multiple(words[1..].to_vec()).await;
                                        match values {
                                            Ok(values) => {
                                                let values: Vec<_> = values.iter().map(|entry| entry.to_string()).collect();
                                                Logger::info_long(&format!("Values of {}", &words[1..].join(", ")), &values.join(", "))
                                            }
                                            Err(err) => Logger::err(err.as_str())
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "set" => {
                        match words.len() {
                            1..=2 => {
                                Logger::warn("Too few parameters");
                            }
                            3 => {
                                let key = words[1];
                                let value = words[2];
                                database.lock().await.set(key, value).await;
                                Logger::info_long(&format!("Set value for {}", key), value);
                            }
                            _ => {
                                let _key = words[1];
                                let _values = &words[2..];
                                Logger::warn("Currently not implemented!");
                            }
                        }
                    }
                    "rm" => {
                        match words.len() {
                            2 => {
                                let key = words[1];
                                database.lock().await.delete(key).await;
                                Logger::info_long("Removed key", key);
                            }
                            _ => {
                                Logger::warn("Too many parameters");
                            }
                        }
                    },
                    "append" => {
                        match words.len() {
                            1..=2 => {
                                Logger::warn("Too few parameters")
                            }
                            3 => {
                                let key = words[1];
                                let value = words[2];
                                database.lock().await.append(key, value).await;
                                Logger::info_long(&format!("Appended value to {}", key), value);
                            }
                            _ => {
                                let _key = words[1];
                                let _values = &words[2..];
                                Logger::warn("Currently not implemented!");
                            }
                        }
                    }
                    "cd" => {
                        match words.len() {
                            2 => {
                                let mut switch = false;
                                for db_type in DB::iter() {
                                    if db_type.to_string() == words[1] {
                                        switch = true;
                                        db = db_type;
                                    }
                                }
                                match switch {
                                    true => Logger::info_long("Switched to database", db.to_string().as_str()),
                                    _    => Logger::warn("Invalid database")
                                }
                            }
                            _ => {
                                Logger::warn("Too many parameters");
                            }
                        }
                    }
                    _ => {
                        Logger::err("Invalid command");
                    }
                }
            }
        });
    });
}

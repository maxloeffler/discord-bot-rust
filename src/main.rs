
use serenity::prelude::{Client, GatewayIntents};
use tokio::runtime::Runtime;
use strum::IntoEnumIterator;

use std::thread;

use utility::traits::Singleton;
use utility::logger::Logger;
use commands::command_manager::CommandManager;
use handler::Handler;
use databases::*;

mod handler;
mod utility;
mod databases;
mod commands;


#[tokio::main]
async fn main() {

    // setup
    let token = setup_db().await;
    let command_handler = CommandManager::new().await;
    let handler = Handler::new(command_handler);

    // start threads
    let intents = GatewayIntents::GUILD_MESSAGES            |
                  GatewayIntents::MESSAGE_CONTENT           |
                  GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");
    spawn_database_thread().await;
    let _ = client.start().await;
}


async fn setup_db() -> String {
    let db = ConfigDB::get_instance().lock().await;
    db.set("token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    db.get("token").await.unwrap()
}

async fn spawn_database_thread() {
    let database = ConfigDB::get_instance();
    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        Logger::info_long("Connected to database", "config");
        runtime.block_on(async {
            loop {
                let input = Logger::input("Enter a command");
                let words = input.split_whitespace().collect::<Vec<&str>>();

                match words[0] {
                    "get" => {
                        match words.len() {
                            1 => {
                                let keys = database.lock().await.get_keys().await;
                                Logger::info_long("Keys", &keys.join(", "));
                            }
                            2 => {
                                let key = words[1];
                                let value = database.lock().await.get(key).await;
                                match value {
                                    Ok(value) => Logger::info_long(&format!("Value of {}", key), &value),
                                    Err(err) => Logger::err(err.as_str())
                                }
                            }
                            _ => {
                                match words[1] {
                                    "all" => {
                                        let values = database.lock().await.get_all(words[2]).await;
                                        match values {
                                            Ok(values) => Logger::info_long(&format!("Values of {}", words[2]), &values.join(", ")),
                                            Err(err) => Logger::err(err.as_str())
                                        }
                                    },
                                    _ => {
                                        let values = database.lock().await.get_multiple(words[1..].to_vec()).await;
                                        match values {
                                            Ok(values) => Logger::info_long(&format!("Values of {}", &words[1..].join(", ")), &values.join(", ")),
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
                    "checkout" => {
                        Logger::warn("Currently not implemented!");
                        continue;
                        match words.len() {
                            2 => {
                                let mut switch = false;
                                for db_type in DB::iter() {
                                    if db_type.to_string() == words[1] {
                                        switch = true;
                                    }
                                }
                                match switch {
                                    true => Logger::info_long("Switched to database", "config"),
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

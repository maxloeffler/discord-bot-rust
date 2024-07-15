
use serenity::prelude::{Client, GatewayIntents};
use tokio::runtime::Runtime;
use strum::IntoEnumIterator;
use colored::*;

use std::thread;
use std::io;

use utility::traits::Singleton;
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

fn logln_warn(message: &str) {
    println!("[{}] {}: {}", "!".red(), "WARNING".red().bold(), message);
}

fn logln_info(info: &str, content: &str) {
    println!("[{}] {}: {}", ">".green(), info.truecolor(128, 128, 128), content);
}

fn log_info(info: &str, content: &str) {
    print!("[{}] {}: {}", ">".green(), info.truecolor(128, 128, 128), content);
}

async fn spawn_database_thread() {
    let database = ConfigDB::get_instance();
    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        logln_info("Connected to database", "config");
        runtime.block_on(async {
            loop {
                log_info("Enter a command", "");
                std::io::Write::flush(&mut io::stdout()).unwrap();

                // read input
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                input = input.trim().to_string();
                let words = input.split_whitespace().collect::<Vec<&str>>();

                match words[0] {
                    "get" => {
                        match words.len() {
                            1 => {
                                let keys = database.lock().await.get_keys().await;
                                logln_info("Keys", &keys.join(", "));
                            }
                            2 => {
                                let key = words[1];
                                let value = database.lock().await.get(key).await;
                                match value {
                                    Ok(value) => logln_info(&format!("Value of {}", key), &value),
                                    Err(err) => logln_warn(err.as_str())
                                }
                            }
                            _ => {
                                match words[1] {
                                    "all" => {
                                        let values = database.lock().await.get_all(words[2]).await;
                                        match values {
                                            Ok(values) => logln_info(&format!("Values of {}", words[2]), &values.join(", ")),
                                            Err(err) => logln_warn(err.as_str())
                                        }
                                    },
                                    _ => {
                                        let values = database.lock().await.get_multiple(words[1..].to_vec()).await;
                                        match values {
                                            Ok(values) => logln_info(&format!("Values of {}", &words[1..].join(", ")), &values.join(", ")),
                                            Err(err) => logln_warn(err.as_str())
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "set" => {
                        match words.len() {
                            1..=2 => {
                                logln_warn("Invalid command");
                            }
                            3 => {
                                let key = words[1];
                                let value = words[2];
                                database.lock().await.set(key, value).await;
                                logln_info(&format!("Set value for {}", key), value);
                            }
                            _ => {
                                let _key = words[1];
                                let _values = &words[2..];
                                logln_warn("Currently not implemented!");
                            }
                        }
                    }
                    "rm" => {
                        match words.len() {
                            2 => {
                                let key = words[1];
                                database.lock().await.delete(key).await;
                                logln_info("Removed key", key);
                            }
                            _ => {
                                logln_warn("Invalid command");
                            }
                        }
                    },
                    "append" => {
                        match words.len() {
                            1..=2 => {
                                logln_warn("Invalid command");
                            }
                            3 => {
                                let key = words[1];
                                let value = words[2];
                                database.lock().await.append(key, value).await;
                                logln_info(&format!("Appended value to {}", key), value);
                            }
                            _ => {
                                let _key = words[1];
                                let _values = &words[2..];
                                logln_warn("Currently not implemented!");
                            }
                        }
                    }
                    "checkout" => {
                        logln_warn("Currently not implemented!");
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
                                    true => logln_info("Switched to database", "config"),
                                    _ => logln_warn("Invalid database")
                                }
                            }
                            _ => {
                                logln_warn("Invalid command");
                            }
                        }
                    }
                    _ => {
                        logln_warn("Invalid command");
                    }
                }
            }
        });
    });
}

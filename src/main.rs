
use serenity::prelude::{Client, GatewayIntents};
use tokio::runtime::Runtime;
use strum::IntoEnumIterator;
use colored::*;

use std::thread;
use std::io;

mod handler;
mod utility;
mod commands;

use utility::database::{Database, DB};
use utility::traits::Singleton;
use commands::command_manager::CommandManager;
use handler::Handler;


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
    let db = Database::get_instance().lock().await;
    db.init().await;
    db.set(DB::Config, "command_prefix", ".").await;
    db.set(DB::Config, "token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    db.get(DB::Config, "token").await.unwrap()
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
    let database = Database::get_instance();
    let mut db = DB::Config;
    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        logln_info("Connected to database", db.to_string().as_str());
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
                                let keys = database.lock().await.get_keys(db.clone()).await;
                                logln_info("Keys", &keys.join(", "));
                            }
                            2 => {
                                let key = words[1];
                                let value = database.lock().await.get(db.clone(), key).await;
                                logln_info(&format!("Value of {}", key), &value.unwrap());
                            }
                            _ => {
                                let values = database.lock().await.get_multiple(db.clone(), &words[1..]).await;
                                logln_info(&format!("Values of {}", &words[1..].join(", ")), &values.unwrap().join(", "));
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
                                database.lock().await.set(db.clone(), key, value).await;
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
                                database.lock().await.delete(db.clone(), key).await;
                                logln_info("Removed key", key);
                            }
                            _ => {
                                logln_warn("Invalid command");
                            }
                        }
                    },
                    "checkout" => {
                        match words.len() {
                            2 => {
                                let mut new_db = db.clone();
                                let mut switch = false;
                                for db in DB::iter() {
                                    if !switch && db.to_string() == words[1] {
                                        new_db = db;
                                        switch = true;
                                    }
                                }
                                match switch {
                                    true => {
                                        db = new_db;
                                        logln_info("Switched to", db.to_string().as_str());
                                    }
                                    false => logln_warn("Invalid database")
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

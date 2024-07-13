
use serenity::prelude::{Client, GatewayIntents};
use tokio::runtime::Runtime;
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
    let intents = GatewayIntents::GUILD_MESSAGES |
                  GatewayIntents::MESSAGE_CONTENT |
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
    db.set(DB::Config, "command_prefix", "xxx").await;
    db.set(DB::Config, "token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U".to_string()
}

async fn spawn_database_thread() {
    let config = Database::get_instance();
    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async {
            println!("[!] Connected to database");
            loop {
                print!("[$] Enter a command: ");
                std::io::Write::flush(&mut io::stdout()).unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                input = input.trim().to_string();
                let words = input.split_whitespace().collect::<Vec<&str>>();
                match words[0] {
                    "get" => {
                        match words.len() {
                            1 => {
                                let keys = config.lock().await.get_keys(DB::Config).await;
                                println!("[$] Keys: {:?}", keys);
                            }
                            2 => {
                                let key = words[1];
                                let value = config.lock().await.get(DB::Config, key).await;
                                println!("[$] Value of key '{}': '{}'", key, value.unwrap());
                            }
                            _ => {
                                let values = config.lock().await.get_multiple(DB::Config, &words[1..]).await;
                                println!("[$] Values of keys '{:?}': '{:?}'", &words[1..], values);
                            }
                        }
                    }
                    "set" => {
                        match words.len() {
                            1..=2 => {
                                println!("[!] Invalid command");
                            }
                            3 => {
                                let key = words[1];
                                let value = words[2];
                                config.lock().await.set(DB::Config, key, value).await;
                                println!("[$] Set key '{}' to value '{}'", key, value);
                            }
                            _ => {
                                let key = words[1];
                                let values = &words[2..];
                                config.lock().await.set(DB::Config, key, values).await;
                            }
                        }
                    }
                    "rm" => {
                        match words.len() {
                            2 => {
                                let key = words[1];
                                config.lock().await.delete(DB::Config, key).await;
                                println!("[$] Removed key '{}'", key);
                            }
                            _ => {
                                println!("[!] Invalid command");
                            }
                        }
                    }
                    _ => {
                        println!("[!] Invalid command");
                    }
                }
            }
        });
    });
}

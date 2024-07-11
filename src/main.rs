
use serenity::prelude::{Client, GatewayIntents};
use tokio::sync::Mutex;
use tokio::runtime::Runtime;
use std::thread;
use std::io;

use std::sync::Arc;

mod handler;
mod utility;
mod commands;

use utility::database::Database;


#[tokio::main]
async fn main() {
    let config = define_config().await;
    let token = config.lock().await.get("token").await;
    let intents = GatewayIntents::GUILD_MESSAGES |
                  GatewayIntents::MESSAGE_CONTENT |
                  GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(token.unwrap(), intents)
        .event_handler(handler::Handler { config: config.clone() })
        .await
        .expect("Error creating client");
    spawn_database_thread(config.clone()).await;
    let _ = client.start().await;
}


async fn define_config() -> Arc<Mutex<Database>> {
    let config = Arc::new(Mutex::new(Database::new( "config" )));
    config.lock().await.set("command_prefix", "xxx").await;
    config.lock().await.set("token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    config
}

async fn spawn_database_thread(config: Arc<Mutex<Database>>) {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
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
                                let keys = config.lock().await.get_keys().await;
                                println!("[$] Keys: {:?}", keys);
                            }
                            2 => {
                                let key = words[1];
                                let value = config.lock().await.get(key).await;
                                println!("[$] Value of key '{}': '{}'", key, value.unwrap());
                            }
                            _ => {
                                let values = config.lock().await.get_multiple(&words[1..]).await;
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
                                config.lock().await.set(key, value).await;
                                println!("[$] Set key '{}' to value '{}'", key, value);
                            }
                            _ => {
                                let key = words[1];
                                let values = &words[2..];
                                config.lock().await.set(key, values).await;
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

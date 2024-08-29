
#![allow(unused_imports)]
use strum::IntoEnumIterator;
use tokio::runtime::Runtime;
use serenity::cache::Settings;
use serenity::prelude::{Client, GatewayIntents};

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
    spawn(database_interface()).await;

    let _ = client.start().await;
}


async fn setup() -> String {
    let config = ConfigDB::get_instance();
    config.set("uptime", &chrono::Utc::now().timestamp().to_string()).await;
    config.set("token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    config.get("token").await.unwrap().to_string()
}

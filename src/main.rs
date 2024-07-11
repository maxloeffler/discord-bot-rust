
use serenity::prelude::{Client, GatewayIntents};
use tokio::sync::Mutex;

use std::sync::Arc;

mod handler;
mod utility;
mod commands;


#[tokio::main]
async fn main() {
    let config = define_config().await;
    let token = config.lock().await.get("token").await;
    let intents = GatewayIntents::GUILD_MESSAGES |
                  GatewayIntents::MESSAGE_CONTENT |
                  GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = Client::builder(token, intents)
        .event_handler(handler::Handler { config: config })
        .await
        .expect("Error creating client");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}


async fn define_config() -> Arc<Mutex<utility::database::Database>> {
    let config = Arc::new(Mutex::new(utility::database::Database::new( "config" )));
    config.lock().await.set("command_prefix", "xxx").await;
    config.lock().await.set("token", "OTk2MzY0MTkzNTg4NTkyNzQw.G8ly6b.Ox24TCFZIQsEc1r-OOXBLbBdWhPIdyc6yKJu0U").await;
    config
}


use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, Ready};
use serenity::prelude::*;
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::utility::message_manager::MessageManager;
use crate::utility::database::Database;


pub struct Handler {
    pub config: Arc<Mutex<Database>>,
}

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self,
                   _ctx: Context,
                   ready: Ready
    ) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self,
                     _ctx: Context,
                     msg: Message
    ) {
        let config = Arc::clone(&self.config);
        let message_manager = MessageManager::new( config, msg ).await;
    }

    async fn message_update(&self,
                            _ctx: Context,
                            _old_if_available: Option<Message>,
                            _new: Option<Message>,
                            _event: MessageUpdateEvent
    ) {
        println!("Message updated");
    }

    async fn message_delete(&self,
                            _ctx: Context,
                            _channel_id: ChannelId,
                            _deleted_message_id: MessageId,
                            _guild_id: Option<GuildId>
    ) {
        println!("Message deleted");
    }

}

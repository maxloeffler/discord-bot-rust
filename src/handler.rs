
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, Ready};
use serenity::prelude::*;
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::utility::message_manager::MessageManager;
use crate::utility::database::Database;
use crate::commands::command_manager::CommandManager;


pub struct Handler {
    pub config: Arc<Mutex<Database>>,
}

impl Handler {
    pub fn clone_config(&self) -> Arc<Mutex<Database>> {
        Arc::clone(&self.config)
    }
}

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self,
                   _ctx: Context,
                   ready: Ready
    ) {
    }

    async fn message(&self,
                     ctx: Context,
                     msg: Message
    ) {
        let message = MessageManager::new( self.clone_config(), ctx, msg ).await;
        if message.is_command() {
            let command_manager = CommandManager::new( self.clone_config(), message ).await;
            command_manager.execute().await;
        }
    }

    async fn message_update(&self,
                            _ctx: Context,
                            _old_if_available: Option<Message>,
                            _new: Option<Message>,
                            _event: MessageUpdateEvent
    ) {
    }

    async fn message_delete(&self,
                            _ctx: Context,
                            _channel_id: ChannelId,
                            _deleted_message_id: MessageId,
                            _guild_id: Option<GuildId>
    ) {
    }

}


use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, Ready};
use serenity::prelude::*;
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::utility::message_manager::MessageManager;
use crate::utility::database::Database;
use crate::commands::command_manager::CommandManager;
use crate::utility::chat_filter::{ChatFilterManager, FilterType};


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

    async fn message(&self, ctx: Context, msg: Message) {

        // parse message
        let message = MessageManager::new( self.clone_config(), ctx, msg ).await;

        // if message pings the bot
        let bot_id = self.clone_config().lock().await.get("bot_id").await;
        if bot_id.is_some() {
            let bot_pings = vec![format!("<@!{}>", bot_id.clone().unwrap()),
                                format!("<@{}>",  bot_id.clone().unwrap())];
            if bot_pings.contains(&message.payload(None, None)) {
                message.reply("Hello!").await;
                return;
            }
        }

        // directly delete messages in the verify channel
        let channel_verify_id = self.clone_config().lock().await.get("channel_verify_id").await;
        if channel_verify_id.is_some() {
            if message.get_channel().get().to_string() == channel_verify_id.unwrap() {
                message.delete().await;
            }
        }

        // check guideline violations
        let chat_filter = ChatFilterManager::new( message.clone() ).filter().await;
        if chat_filter.filter == FilterType::Fine || message.is_trial().await || message.get_author().bot {

            // execute command
            if message.is_command() {
                let command_manager = CommandManager::new( self.clone_config(), message ).await;
                command_manager.execute().await;
            }

        } else {

            // message.delete().await;
            println!("Message deleted because it contained '{}'", chat_filter.context);
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

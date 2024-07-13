
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, Ready};
use serenity::prelude::*;

use crate::utility::message_manager::MessageManager;
use crate::utility::database::{Database, DB};
use crate::commands::command_manager::CommandManager;
use crate::utility::chat_filter::{ChatFilterManager, FilterType};
use crate::utility::traits::Singleton;


pub struct Handler {
    command_manager: CommandManager,
}

impl Handler {

    pub fn new(command_manager: CommandManager) -> Handler {
        Handler {
            command_manager,
        }
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
        let message = MessageManager::new(ctx, msg).await;

        // if message pings the bot
        let bot_id = Database::get_instance().lock().await.get(DB::Config, "bot_id").await;
        if bot_id.is_some() {
            let bot_pings = vec![format!("<@!{}>", bot_id.clone().unwrap()),
                                format!("<@{}>",  bot_id.clone().unwrap())];
            if bot_pings.contains(&message.payload(None, None)) {
                message.reply("Hello!").await;
                return;
            }
        }

        // directly delete messages in the verify channel
        let channel_verify_id = Database::get_instance().lock().await.get(DB::Config, "channel_verify_id").await;
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
                self.command_manager.execute(message).await;
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

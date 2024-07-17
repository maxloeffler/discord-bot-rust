
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent};
use serenity::prelude::*;

use crate::commands::command_manager::CommandManager;
use crate::utility::message_manager::MessageManager;
use crate::utility::resolver::Resolver;
use crate::utility::chat_filter::{ChatFilterManager, FilterType};
use crate::utility::traits::Singleton;
use crate::databases::*;


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

    async fn message(&self, ctx: Context, msg: Message) {

        // parse message
        let resolver = Resolver::new( ctx.clone(), msg.guild_id );
        let message = MessageManager::new(resolver, msg).await;

        // if message pings the bot
        let bot_id = ConfigDB::get_instance().lock().await
            .get("bot_id").await.unwrap().to_string();
        let bot_pings = vec![format!("<@!{}>", bot_id.clone()),
                             format!("<@{}>",  bot_id.clone())];
        if bot_pings.contains(&message.payload(None, None)) {
            message.reply("Hello!").await;
            return;
        }

        // directly delete messages in the verify channel
        let channel_verify = ConfigDB::get_instance().lock().await
            .get("channel_verify").await.unwrap().to_string();
        if message.get_channel().get().to_string() == channel_verify {
            message.delete().await;
        }

        // check guideline violations
        let chat_filter = ChatFilterManager::new( message.clone() ).filter().await;
        if chat_filter.filter == FilterType::Fine
            || message.is_trial().await
            || message.get_author().bot {

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

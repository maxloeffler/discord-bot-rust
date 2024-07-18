
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, CreateEmbedFooter};
use serenity::prelude::*;

use crate::commands::command_manager::CommandManager;
use crate::utility::message_manager::MessageManager;
use crate::utility::log_builder::LogBuilder;
use crate::utility::resolver::Resolver;
use crate::utility::chat_filter::{ChatFilterManager, FilterType};
use crate::utility::traits::{Singleton, ToMessage};
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

            // automatically delete message and warn
            if chat_filter.filter != FilterType::Fine {

                message.delete().await;

                // warn user
                let warn_message = format!("<@{}>, you have been **automatically warned** `>` {}",
                    message.get_author().id.to_string(),
                    chat_filter.context);
                message.reply(warn_message.to_message()).await;
                let database_reason = format!("Automatically warned ('{}')", chat_filter.context);
                WarningsDB::get_instance().lock().await
                    .append(&message.get_author().id.to_string(), &database_reason).await;

            }
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
                            ctx: Context,
                            channel_id: ChannelId,
                            deleted_message_id: MessageId,
                            guild_id: Option<GuildId>
    ) {

        // get all excluded channels
        let channel_protected_log: Vec<_> = ConfigDB::get_instance().lock().await
            .get_multiple(vec!["channel_messagelogs", "channel_admin", "channel_headmod"]).await.unwrap()
            .iter()
            .map(|entry| entry.value.to_string())
            .collect();

        // do not log messages from protected channels
        if channel_protected_log.contains(&channel_id.to_string()) {
            return;
        }

        // obtain Message object
        let resolver = Resolver::new(ctx.clone(), guild_id);
        let channel_messagelogs_id = channel_protected_log[0].clone();
        let channel_messagelogs = resolver.resolve_channel(channel_messagelogs_id).await.unwrap();
        let message = resolver.resolve_message(channel_id, deleted_message_id).await;

        // cannot continue if message cannot be resolved
        if let Some(message) = message {

            let message = MessageManager::new(resolver.clone(), message).await;

            // do not log messages from bots
            if message.get_author().bot {
                return;
            }

            let name = resolver.resolve_name(message.get_author());
            let log_builder = LogBuilder::new(message.clone())
                .title(&format!("{}'s Message Deleted", name))
                .description("Message Information")
                .labeled_timestamp("Sent", message.get_timestamp())
                .labeled_timestamp("Deleted", chrono::Utc::now().timestamp())
                .channel();

            // split message content into chunks of 1024 because of Discord embed field limit
            let chars = message.payload(None, None)
                .chars().collect::<Vec<_>>();
            let chunks = chars
                .chunks(1024)
                .collect::<Vec<_>>();
            let _ = chunks
                .iter()
                .enumerate()
                .for_each(|(i, chunk)| {
                    let content = chunk.iter().collect::<String>();
                    log_builder.clone().arbitrary(
                        &format!("Message Content ({}/{})", i + 1, chunks.len()),
                        &content);
                });

            // add additional fields
            let mut log_message = log_builder.build().await
                .footer(CreateEmbedFooter::new(
                    format!("User ID: {}", message.get_author().id)));
            message.get_attachments().await.iter().for_each(|attachment| {
                log_message = log_message.clone().image(attachment.url.clone());
            });

            // log message
            let _ = channel_messagelogs.send_message(&resolver.http(), log_message.to_message()).await;
        }
    }
}

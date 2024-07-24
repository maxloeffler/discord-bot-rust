
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::model::guild::Member;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, CreateEmbedFooter, EditChannel};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use difference::{Difference, Changeset};

use std::str::FromStr;

use crate::commands::command_manager::CommandManager;
use crate::utility::*;
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

    async fn ready(&self, ctx: Context, _ready: Ready) {

        #[cfg(feature = "debug")]
        Logger::info("Bot is ready!");

        let main_guild = ConfigDB::get_instance().lock().await
            .get("guild_main").await.unwrap().to_string();
        let guild_id = GuildId::from_str(&main_guild).unwrap();
        let resolver = Resolver::new(ctx, Some(guild_id));

        TicketHandler::get_instance().lock().await
            .init(&resolver).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {

        // parse message
        let resolver = Resolver::new(ctx, msg.guild_id);
        let message = MessageManager::new(resolver, msg).await;

        // if message pings the bot
        let bot_id = ConfigDB::get_instance().lock().await
            .get("bot_id").await.unwrap().to_string();
        let bot_pings = vec![format!("<@!{}>", &bot_id),
                             format!("<@{}>",  &bot_id)];
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
        let chat_filter = ChatFilterManager::new(&message).filter().await;
        if chat_filter.filter == FilterType::Fine
            || message.is_trial().await
            || message.get_author().bot {

            // execute command
            #[cfg(feature = "commands")]
            if message.is_command() {
                self.command_manager.execute(&message).await;
            }

        } else {

            // automatically delete message and warn
            #[cfg(feature = "auto_moderation")]
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

    async fn guild_member_addition(&self,
                                   ctx: Context,
                                   new_member: Member
    ) {
        // get guild
        let guild_id = new_member.guild_id;
        let resolver = Resolver::new(ctx, Some(guild_id));
        let guild = resolver.resolve_guild(guild_id).await;

        // get welcome channel
        let channel_id = ConfigDB::get_instance().lock().await
            .get("channel_welcome").await.unwrap().to_string();
        let channel = resolver.resolve_channel(channel_id).await;

        // update channel name
        if guild.is_some() && channel.is_some() {
            let edit = EditChannel::new()
                .name(&format!("Kalopsians: {}", guild.unwrap().member_count));
            let _ = channel.unwrap().edit(&resolver.http(), edit).await;
        }
    }

    #[cfg(feature = "auto_moderation")]
    async fn guild_member_removal(&self,
                                ctx: Context,
                                guild_id: GuildId,
                                user: User,
                                _member_data_if_available: Option<Member>,
    ) {
        let resolver = Resolver::new(ctx, Some(guild_id));
        let is_muted = resolver.has_role(&user, "Muted").await;
        if is_muted {
            AutoModerator::get_instance().lock().await
                .perform_ban(&resolver, &user, "Left while muted.".to_string()).await;
        }
    }


    #[cfg(feature = "message_logs")]
    async fn message_update(&self,
                            ctx: Context,
                            old_if_available: Option<Message>,
                            new: Option<Message>,
                            event: MessageUpdateEvent
    ) {

        // get all excluded channels
        let channel_protected_log: Vec<_> = ConfigDB::get_instance().lock().await
            .get_multiple(vec!["channel_messagelogs", "channel_admin", "channel_headmod"]).await.unwrap()
            .iter()
            .map(|entry| entry.value.to_string())
            .collect();

        // do not log messages from protected channels
        if channel_protected_log.contains(&event.channel_id.to_string()) {
            return;
        }

        if let Some(new_message) = new {

            let resolver = Resolver::new(ctx, event.guild_id);
            let message = &MessageManager::new(resolver, new_message).await;

            // do not log messages from bots
            if message.get_author().bot {
                return;
            }

            let name = message.resolve_name();
            let log_builder = message.get_log_builder()
                .title(&format!("{}'s Message Edited", name))
                .description("Message Information")
                .labeled_timestamp("Sent", message.get_timestamp())
                .labeled_timestamp("Edited", chrono::Utc::now().timestamp())
                .channel();

            let diff_string = match old_if_available {
                Some(old_message) => {
                    let changeset = Changeset::new(
                        &old_message.content,
                        &message.payload(None, None),
                        " ");
                    let mut diff = vec!["```diff".to_string()];
                    changeset.diffs.iter().for_each(|difference| {
                        let line = match difference {
                            Difference::Same(text) => text.to_string(),
                            Difference::Add(text) => format!("+ {}", text),
                            Difference::Rem(text) => format!("- {}", text),
                        };
                        diff.push(line);
                    });
                    diff.push("```".to_string());
                    diff.join("\n")
                },
                None => "Content of original message is not available.".to_string(),
            };

            // add additional fields
            let mut log_message = log_builder.build().await
                .field("Message Content", diff_string, true)
                .footer(CreateEmbedFooter::new(
                    format!("User ID: {}", message.get_author().id)));
            for attachment in message.get_attachments().await.iter() {
                log_message = log_message.image(attachment.url.clone());
            }
 
            // send log message
            let channel_id = channel_protected_log[0].clone();
            let channel = message.get_resolver()
                .resolve_channel(channel_id).await.unwrap();
            let _ = channel.send_message(
                &message.get_resolver().http(),
                log_message.to_message()).await;
        }

    }

    #[cfg(feature = "message_logs")]
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
        let resolver = Resolver::new(ctx, guild_id);
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
            let log_builder = message.get_log_builder()
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
            for attachment in message.get_attachments().await.iter() {
                log_message = log_message.image(attachment.url.clone());
            }

            // log message
            let _ = channel_messagelogs.send_message(&resolver.http(), log_message.to_message()).await;
        }
    }
}

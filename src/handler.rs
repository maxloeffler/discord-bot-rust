
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::model::guild::Member;
use serenity::all::{ChannelId, MessageId, GuildId, MessageUpdateEvent, CreateEmbedFooter, EditChannel};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use difference::{Difference, Changeset};
use futures::stream::StreamExt;

use std::sync::Arc;
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

        let main_guild = ConfigDB::get_instance()
            .get("guild_main").await.unwrap().to_string();
        let guild_id = GuildId::from_str(&main_guild).unwrap();
        let resolver = Resolver::new(ctx, Some(guild_id));

        #[cfg(feature = "tickets")]
        TicketHandler::get_instance()
            .init(&resolver).await;

        spawn(periodic_checks(resolver.clone())).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {

        // parse message
        let resolver = Resolver::new(ctx, msg.guild_id);
        let mut message = Arc::new(MessageManager::new(resolver, msg).await);

        // if message pings the bot
        let bot_id = &ConfigDB::get_instance().get("bot_id").await.unwrap().to_string();
        let bot_pings = vec![format!("<@!{}>", bot_id), format!("<@{}>",  bot_id)];
        if bot_pings.contains(&message.payload(None, None)) {
            let prefix = ConfigDB::get_instance().get("command_prefix").await.unwrap().to_string();
            message = message.spoof(format!("{}about", prefix)).await.into();
        }

        // directly delete messages in the verify channel
        let channel_verify: ChannelId = ConfigDB::get_instance()
            .get("channel_verify").await.unwrap().into();
        if message.get_channel() == channel_verify {
            message.delete().await;
        }

        // check if author is afk
        let author = &message.get_author();
        let author_id = &author.id.to_string();
        let author_afk = AfkDB::get_instance().get(author_id).await;
        if author_afk.is_ok() {
            let embed = MessageManager::create_embed(|embed| {
                embed.description("Removed your afk.")
            }).await;
            let _ = message.reply_temporary(embed).await;
            AfkDB::get_instance().delete(&author_id).await;
        }

        // check if message mentions an afk user
        let mentions = message.get_mentions().await;
        futures::stream::iter(mentions)
            .for_each_concurrent(None, |mention| {
                let message = Arc::clone(&message);
                async move {
                    let mention_afk = AfkDB::get_instance().get(&mention.to_string()).await;
                    if let Ok(afk) = mention_afk {
                        let embed = MessageManager::create_embed(|embed| {
                            embed.description(
                                &format!("<@{}> is currently afk `>` {}",
                                    mention.to_string(),
                                    afk.to_string()))
                        }).await;
                        message.reply_temporary(embed).await;
                    }
                }}).await;

        // check guideline violations
        let filter = ChatFilter::get_instance().apply(&message).await;
        if filter.filter_type == FilterType::Fine || message.is_trial().await || author.bot {

            // execute command
            #[cfg(feature = "commands")]
            if message.is_command() {
                self.command_manager.execute(&message).await;
            }

        } else {

            // automatically delete message and warn
            #[cfg(feature = "auto_moderation")]
            if filter.filter_type != FilterType::Fine {

                message.delete().await;
                AutoModerator::get_instance()
                    .perform_warn(&message, &author, filter.filter_type.to_string(), filter.context).await;
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
        let guild = resolver.resolve_guild(None).await;

        // get member count channel
        let channel: ChannelId = ConfigDB::get_instance()
            .get("channel_member_count").await.unwrap().into();

        // update channel name
        if guild.is_some() {
            let edit = EditChannel::new()
                .name(&format!("Kalopsians: {}", guild.unwrap().member_count));
            let _ = channel.edit(&resolver, edit).await;
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
            AutoModerator::get_instance()
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
        let channel_protected_log: Vec<_> = ConfigDB::get_instance()
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
            let channel = channel_protected_log[0].clone();
            let channel = ChannelId::from_str(&channel).unwrap();
            let _ = channel.send_message(message, log_message.to_message()).await;
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
        let channel_protected_log: Vec<_> = ConfigDB::get_instance()
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
        let channel_messagelogs = channel_protected_log[0].clone();
        let channel_messagelogs = ChannelId::from_str(&channel_messagelogs).unwrap();
        let message = resolver.resolve_message(channel_id, deleted_message_id).await;

        // cannot continue if message cannot be resolved
        if let Some(message) = message {

            let message = MessageManager::new(resolver.clone(), message).await;

            // do not log messages from bots
            if message.get_author().bot {
                return;
            }

            let name = resolver.resolve_name(message.get_author());
            let mut log_builder = message.get_log_builder()
                .title(&format!("{}'s Message Deleted", name))
                .description("Message Information")
                .labeled_timestamp("Sent", message.get_timestamp())
                .labeled_timestamp("Deleted", chrono::Utc::now().timestamp())
                .channel();

            // split message content into chunks of 1024 because of Discord embed field limit
            let chars = message.words.join(" ").chars().collect::<Vec<_>>();
            let chunks = chars.chunks(1024).collect::<Vec<_>>();
            for chunk in chunks.into_iter() {
                let content = chunk.into_iter().collect::<String>();
                log_builder = log_builder.arbitrary("Message Content", &content);
            }

            // add additional fields
            let mut log_message = log_builder.build().await
                .footer(CreateEmbedFooter::new(
                    format!("User ID: {}", message.get_author().id)));
            for attachment in message.get_attachments().await.iter() {
                log_message = log_message.image(attachment.url.clone());
            }

            // log message
            let _ = channel_messagelogs.send_message(resolver, log_message.to_message()).await;
        }
    }
}


use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct WarnCommand;

impl Command for WarnCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["warn".to_string()]
        )
            .add_required("user")
            .add_optional("reason")
            .example("@BadBoy being bad")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let mentions = message.get_mentions().await;

                // check if a user is mentioned
                if mentions.len() == 0 {
                    self.invalid_usage(params).await;
                    return;
                }

                // check if the user is a moderator
                let resolver = message.get_resolver();
                let target = resolver.resolve_user(mentions[0]).await;

                if target.is_none() {
                    message.reply_failure("User not found.").await;
                    return
                }
                let target = &target.unwrap();

                if resolver.is_trial(&target).await {
                    message.reply_failure("You can't warn a moderator.").await;
                    return;
                }

                // obtain the reason
                let mut reason = message.payload_without_mentions(None, None);
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                // log to database
                let log = ModLog::new(
                    message.get_author().id.to_string(),
                    reason.clone(),
                );
                WarningsDB::get_instance()
                    .append(&target.id.to_string(), &log.into()).await;

                // create embed
                let embed = MessageManager::create_embed(|embed|
                    embed
                        .title(&format!("Warning"))
                        .description(&format!("You have been warned for `>` {}", reason))
                        .color(0xff0000)
                ).await;
                let _ = message.reply(format!("<@{}>", target.id)).await;
                let _ = message.reply(embed).await;

                // log to mod logs
                let log_message = message.get_log_builder()
                    .title("[WARNING]")
                    .target(&target)
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .arbitrary("Reason", &reason)
                    .timestamp()
                    .build().await;
                let modlogs: ChannelId = ConfigDB::get_instance()
                    .get("channel_modlogs").await.unwrap().into();
                let _ = modlogs.send_message(message, log_message.to_message()).await;

                // check if the user has been warned too many times
                #[cfg(feature = "auto_moderation")]
                AutoModerator::get_instance()
                    .check_warnings(message, &target).await;

            }
        )
    }

}



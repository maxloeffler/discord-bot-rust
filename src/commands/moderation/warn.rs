
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct WarnCommand;

impl Command for WarnCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["warn".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;
                let mentions = message.get_mentions().await;

                // check if a user is mentioned
                if mentions.len() == 0 {
                    message.reply_failure("You need to mention someone to warn them.").await;
                    return;
                }

                // check if the user is a moderator
                let resolver = message.get_resolver();
                let target = &resolver.resolve_user(mentions[0]).await.unwrap();
                if resolver.is_trial(&target).await {
                    message.reply_failure("You can't warn a moderator.").await;
                    return;
                }

                // obtain the reason
                let mut reason = message.payload_without_mentions(None, None).await;
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                // log to database
                let log = ModLog {
                    member_id: target.id.to_string(),
                    staff_id: message.get_author().id.to_string(),
                    reason: reason.clone(),
                };
                WarningsDB::get_instance().lock().await
                    .append(&target.id.to_string(), &log.into()).await;
                let embed = MessageManager::create_embed(|embed|
                    embed
                        .title(&format!("Warning"))
                        .description(&format!("You have been warned for `>` {}", reason))
                        .color(0xff0000)
                ).await;
                let _ = message.reply(format!("<@{}>", target.id)).await;
                let _ = message.reply(embed).await;

                // log to mod logs
                let channel_modlogs_id = ConfigDB::get_instance().lock().await
                    .get("channel_modlogs").await.unwrap().to_string();
                let channel_modlogs = resolver.resolve_channel(channel_modlogs_id).await.unwrap();
                let log_message = message.get_log_builder()
                    .title("[WARNING]")
                    .description(&format!("<@{}> has been warned", target.id))
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .arbitrary("Reason", &reason)
                    .timestamp()
                    .build().await;
                let _ = channel_modlogs.send_message(resolver, log_message.to_message()).await;

                // check if the user has been warned too many times
                #[cfg(feature = "auto_moderation")]
                AutoModerator::get_instance().lock().await
                    .check_warnings(resolver, &target).await;

            }
        )
    }

}



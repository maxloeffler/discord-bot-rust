
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct MuteCommand;

impl Command for MuteCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "mute".to_string(),
        ])
            .add_required("user")
            .add_optional("reason")
            .example("mute @BadBoy continuously being bad")
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
                let target = &resolver.resolve_user(mentions[0]).await.unwrap();
                if resolver.is_trial(&target).await {
                    message.reply_failure("You can't mute a moderator.").await;
                    return;
                }

                // check if member is already muted
                let role_muted = &resolver.resolve_role("Muted").await.unwrap()[0].id;
                let member = resolver.resolve_member(&target).await.unwrap();
                if member.roles.contains(role_muted) {
                    message.reply_failure(&format!("<@{}> is already muted.", target.id.to_string())).await;
                    return;
                }

                member.add_role(&resolver, role_muted).await.unwrap();

                // obtain the reason
                let mut reason = message.payload_without_mentions(None, None);
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                // log mute to database
                let log = ModLog {
                    member_id: target.id.to_string(),
                    staff_id: message.get_author().id.to_string(),
                    reason: reason.clone()
                };
                MutesDB::get_instance().lock().await
                    .append(&target.id.to_string(), &log.into()).await;

                // log mute to mod logs
                let log_message = message.get_log_builder()
                    .title("[MUTE]")
                    .description(&format!("<@{}> has been muted", target.id))
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .arbitrary("Reason", &reason)
                    .timestamp()
                    .build().await;
                let modlogs: ChannelId = ConfigDB::get_instance().lock().await
                    .get("channel_modlogs").await.unwrap().into();
                let _ = modlogs.send_message(resolver, log_message.to_message()).await;

                message.reply_success().await;

                // check for active flags
                #[cfg(feature = "auto_moderation")]
                AutoModerator::get_instance().lock().await
                    .check_mutes(resolver, target).await;
            }
        )
    }

}



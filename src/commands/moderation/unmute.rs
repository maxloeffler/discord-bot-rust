
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct UnmuteCommand;

impl Command for UnmuteCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["unmute".to_string()]
        )
            .add_required("user")
            .new_usage()
            .add_required("user")
            .add_constant("-flag", false)
            .add_required("reason")
            .add_constant("-monthly", false)
            .example("@BadBoy -flag")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                // check if the user is a moderator
                let resolver = message.get_resolver();
                if resolver.is_trial(&target).await {
                    message.reply_failure("You can't mute a moderator.").await;
                    return;
                }

                // check if member is already muted
                let role_muted = &resolver.resolve_role("Muted").await.unwrap()[0].id;
                let member = resolver.resolve_member(&target).await.unwrap();
                if !member.roles.contains(role_muted) {
                    message.reply_failure(&format!("<@{}> is not muted.", target.id.to_string())).await;
                    return;
                }

                member.remove_role(&resolver, role_muted).await.unwrap();
                let mut builder = message.get_log_builder()
                    .title("[UNMUTE]")
                    .target(&target)
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .timestamp();

                let last_mute = MutesDB::get_instance()
                    .get_last(&target.id.to_string(), 1).await.unwrap();

                // obtain the reason
                let mut reason = message.payload_without_mentions(None, None);
                if reason.is_empty() {
                    match last_mute.is_empty() {
                        true  => reason = "No reason provided".to_string(),
                        false => reason = last_mute[0].reason.clone(),
                    }
                }

                // log mute to database
                let log = ModLog::new(
                    message.get_author().id.to_string(),
                    reason.clone()
                );
                UnmutesDB::get_instance()
                    .append(&target.id.to_string(), &log.into()).await;

                // flag member if specified
                if message.has_parameter("flag") {

                    let monthly = message.has_parameter("monthly");

                    // log flag to database
                    let log = FlagLog::new(
                        message.get_author().id.to_string(),
                        reason.clone(),
                        monthly
                    );
                    FlagsDB::get_instance()
                        .append(&target.id.to_string(), &log.into()).await;

                    let timestamp_now = chrono::Utc::now().timestamp();
                    builder = builder.labeled_timestamp("Flag Until", match monthly {
                        true  => timestamp_now + (30 * 24 * 60 * 60),
                        false => timestamp_now + (7  * 24 * 60 * 60)
                    });

                }

                // log to mod logs
                let log = builder.build().await.to_message();
                let modlogs: ChannelId = ConfigDB::get_instance()
                    .get("channel_modlogs").await.unwrap().into();
                let _ = modlogs.send_message(message, log).await;

                message.reply_success().await;
            }
        )
    }

}



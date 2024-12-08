
use serenity::model::permissions::Permissions;
use serenity::all::ChannelId;
use serenity::builder::EditChannel;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;
use std::str::FromStr;

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct LockCommand;

impl Command for LockCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["lock".to_string(), "lock-user".to_string()]
        )
            .add_required("user")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.clone().unwrap();

                if message.get_resolver().is_trial(&target).await {
                    message.reply_failure("You cannot lock a moderator!").await;
                    return;
                }

                if let Some(channel) = message.resolve_guild_channel().await {

                    // revoke member's permissions
                    let handler = PermissionHandler::new(&message.get_resolver(), &channel);
                    handler.deny_member(
                        vec![&Permissions::SEND_MESSAGES, &Permissions::VIEW_CHANNEL],
                        &target.id)
                    .await;

                    // log user lock to modlogs
                    let embed = message.get_log_builder()
                        .title("[LOCK]")
                        .target(target)
                        .staff()
                        .user(target)
                        .channel()
                        .timestamp()
                        .build().await;
                    let modlogs: ChannelId = ConfigDB::get_instance()
                        .get("channel_modlogs").await.unwrap().into();
                    let _ = modlogs.send_message(message, embed.to_message()).await;

                    message.reply_success().await;
                }
            }
        )
    }

}

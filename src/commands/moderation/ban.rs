
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct BanCommand;

impl Command for BanCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_mod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["ban".to_string()]
        )
            .add_required("user")
            .add_optional("reason")
            .example("@JuicyJuggler we could not handle you anymore")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                // obtain the reason
                let mut reason = message.payload_without_mentions(None, None);
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                let resolver = message.get_resolver();
                if resolver.is_trial(&target).await {
                    message.reply_failure("You can't ban a moderator.").await;
                    return;
                }

                let member = resolver.resolve_member(&target).await;
                if let Some(member) = member {

                    // ban the user
                    let role_muted = &resolver.resolve_role("Muted").await.unwrap()[0];
                    let _ = member.remove_role(&resolver, role_muted.id).await;
                    let _ = member.ban_with_reason(&resolver, 0, &reason).await;

                    // log ban to database
                    let log = ModLog::new(
                        message.get_author().id.to_string(),
                        reason.clone()
                    );
                    BansDB::get_instance()
                        .append(&target.id.to_string(), &log.into()).await;

                    // log ban to mod logs
                    let log_message = message.get_log_builder()
                        .title("[BAN]")
                        .description(&format!("<@{}> has been banned", target.id))
                        .target(&target)
                        .color(0xff8200)
                        .staff()
                        .user(&target)
                        .arbitrary("Reason", &reason)
                        .timestamp()
                        .build().await;
                    let modlogs: ChannelId = ConfigDB::get_instance()
                        .get("channel_modlogs").await.unwrap().into();
                    let _ = modlogs.send_message(resolver, log_message.to_message()).await;

                    // inform member of the ban and how to appeal
                    let guild = resolver.resolve_guild(None).await.unwrap();
                    let notify_message = message.get_log_builder()
                        .title("You've been banned!")
                        .description(&format!("You have been banned from {} for \"{}\"\nYou can appeal your ban [here](https://dyno.gg/form/f2f3a893) if you believe that we made a mistake!",
                            guild.name,
                            reason))
                        .target(&target)
                        .no_thumbnail()
                        .color(0xff0000)
                        .build().await;
                    let sent = target.direct_message(resolver, notify_message.to_message()).await;

                    match sent {
                        Ok(_)  => message.reply_success().await,
                        Err(_) => {
                            let _ = message.reply("Notice: I couldn't send a DM to the user.").await;
                        }
                    };
                }
            }
        )
    }

}


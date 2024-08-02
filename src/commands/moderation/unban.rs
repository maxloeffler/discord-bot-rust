
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct UnbanCommand;

impl Command for UnbanCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_mod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "unban".to_string(),
        ])
            .add_required("user-id")
            .add_optional("reason")
            .example("unban @RecoveredRobin has promised to behave")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let resolver = &message.get_resolver();
                let mentions = message.get_mentions().await;

                if mentions.len() != 1 {
                    self.invalid_usage(params).await;
                    return;
                }
                let target_id = mentions[0];

                // try to resolve target
                let target = resolver.resolve_user(target_id).await;
                let name = match target {
                    Some(ref target) => resolver.resolve_name(&target),
                    None => target_id.to_string(),
                };

                // obtain the reason
                let mut reason = message.payload_without_mentions(None, Some(vec![target_id.to_string()])).await;
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                if let Some(guild_id) = message.get_guild() {

                    // unban the user
                    let _ = guild_id.unban(&message, target_id).await;

                    // clear databases
                    WarningsDB::get_instance().lock().await
                        .delete(&target_id.to_string()).await;
                    MutesDB::get_instance().lock().await
                        .delete(&target_id.to_string()).await;
                    FlagsDB::get_instance().lock().await
                        .delete(&target_id.to_string()).await;

                    // get reason of last ban
                    let last_ban = BansDB::get_instance().lock().await
                        .get_last(&target_id.to_string(), 1).await;
                    let ban_reason = match last_ban {
                        Ok(bans) => {
                            match bans.is_empty() {
                                true  => "No reason provided.".to_string(),
                                false => {
                                    let log: ModLog = (&bans[0]).into();
                                    log.reason
                                }
                            }
                        },
                        Err(_) => "No reason provided.".to_string()
                    };

                    // log unban to mod logs
                    let channel_modlogs_id = ConfigDB::get_instance().lock().await
                        .get("channel_modlogs").await.unwrap().to_string();
                    let channel_modlogs = ChannelId::from_str(&channel_modlogs_id).unwrap();
                    let target = match target {
                        Some(ref target) => target,
                        None => {
                            let bot_id = ConfigDB::get_instance().lock().await
                                .get("bot_id").await.unwrap().to_string();
                            &resolver.resolve_user(UserId::from_str(&bot_id).unwrap()).await.unwrap()
                        }
                    };
                    let embed = message.get_log_builder()
                        .title("[UNBAN]")
                        .target(target)
                        .description(&format!("{} has been unbanned", name))
                        .color(0xff8200)
                        .staff()
                        .arbitrary("Ban Reason", &ban_reason)
                        .arbitrary("Unban Reason", &reason)
                        .timestamp()
                        .build().await;
                    let _ = channel_modlogs.send_message(resolver, embed.to_message()).await;

                    // inform member of their unban
                    let guild = resolver.resolve_guild(None).await.unwrap();
                    let notify_message = message.get_log_builder()
                        .title("You've been unbanned!")
                        .description(&format!(
                            "You have been unbanned from {}\nFeel free to join us again: discord.gg/vent!",
                            guild.name))
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


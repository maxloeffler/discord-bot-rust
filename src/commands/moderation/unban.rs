
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{CommandType, Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct UnbanCommand;

impl Command for UnbanCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_mod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["unban".to_string()]
        )
            .add_required("user-id")
            .add_optional("reason")
            .example("@RecoveredRobin has promised to behave")
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
                let mut reason = message.payload_without_mentions(None, Some(vec![target_id.to_string()]));
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                if let Some(guild_id) = message.get_guild() {

                    // unban the user
                    let _ = guild_id.unban(&message, target_id).await;

                    // clear databases
                    WarningsDB::get_instance().delete(&target_id.to_string()).await;
                    MutesDB::get_instance().delete(&target_id.to_string()).await;
                    FlagsDB::get_instance().delete(&target_id.to_string()).await;

                    // get reason of last ban
                    let last_ban = BansDB::get_instance()
                        .get_last(&target_id.to_string(), 1).await;
                    let ban_reason = match last_ban {
                        Ok(bans) => {
                            match bans.is_empty() {
                                true  => "No reason provided.".to_string(),
                                false => bans[0].reason.clone()
                            }
                        },
                        Err(_) => "No reason provided.".to_string()
                    };

                    // log unban to mod logs
                    let target = match target {
                        Some(ref target) => target,
                        None => {
                            let bot_id: UserId = ConfigDB::get_instance()
                                .get("bot_id").await.unwrap().into();
                            &resolver.resolve_user(bot_id).await.unwrap()
                        }
                    };
                    let embed = message.get_log_builder()
                        .title("[UNBAN]")
                        .description(&format!("{} has been unbanned", name))
                        .color(0xff8200)
                        .staff()
                        .arbitrary("Ban Reason", &ban_reason)
                        .arbitrary("Unban Reason", &reason)
                        .timestamp()
                        .build().await;
                    let unbanlogs: ChannelId = ConfigDB::get_instance()
                        .get("channel_unbanlogs").await.unwrap().into();
                    let _ = unbanlogs.send_message(message, embed.to_message()).await;

                    // inform member of their unban
                    let guild = resolver.resolve_guild(None).await.unwrap();
                    let notify_message = message.get_log_builder()
                        .title("You've been unbanned!")
                        .description(&format!(
                            "You have been unbanned from {}\nFeel free to join us again [here](discord.gg/vent)!",
                            guild.name))
                        .target(&target)
                        .no_thumbnail()
                        .color(0xff0000)
                        .build().await;
                    let sent = target.dm(resolver, notify_message.to_message()).await;

                    match sent {
                        Ok(_)  => message.reply_success().await,
                        Err(_) => {
                            let embed = MessageManager::create_embed(|embed| {
                                embed
                                    .title("Notice")
                                    .description("I could not send a DM to the user.")
                            }).await;
                            let _ = message.reply(embed).await;
                        }
                    };
                }
            }
        )
    }

}


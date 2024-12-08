
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{CommandType, Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct CheckBanCommand;

impl Command for CheckBanCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["check-ban".to_string(), "checkban".to_string(), "bans".to_string()]
        )
            .add_required("user-id")
            .example("996364193588592740")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target_id = &message.payload(None, None);

                let id = target_id.parse::<u64>();
                if id.is_err() {
                    self.invalid_usage(params).await;
                    return;
                }

                // try to resolve target
                let target = message.get_resolver()
                    .resolve_user(UserId::from_str(target_id).unwrap()).await;
                let name = match target {
                    Some(ref target) => message.get_resolver().resolve_name(&target),
                    None => target_id.clone(),
                };

                // get bans
                if let Some(guild) = message.get_resolver().resolve_guild(None).await {
                    let bans = guild.bans(message, None, None).await;
                    if let Ok(bans) = bans {

                        let current_ban = bans.into_iter()
                            .filter(|ban| ban.user.id.to_string() == *target_id)
                            .next();

                        // collect information abount bans
                        let mut all_bans = match current_ban {
                            Some(ban) => {
                                vec![(
                                    ban.reason.unwrap_or("No reason provided.".to_string()),
                                    None
                                )]
                            },
                            None => Vec::new(),
                        };

                        // add bans the bot has issued that have been revoked by now
                        if target.is_some() {
                            if let Ok(recorded_bans) = BansDB::get_instance().get_all(target_id).await {

                                recorded_bans.into_iter()
                                    .for_each(|ban| {

                                        // NOTE: ban reasons are unique!
                                        // Discord does not store the timestamp of the current ban
                                        // but the bot logs do. Therefore, if we find the log of
                                        // the current ban in the bot logs, we can use the
                                        // timestamp that is stored there.
                                        let index = all_bans.iter().position(|(reason, _)| reason == &ban.reason);
                                        if let Some(i) = index {
                                            all_bans[i].1 = Some(ban.timestamp);
                                        }

                                        // Regularely append ban information to the list
                                        else {
                                            all_bans.push((ban.reason, Some(ban.timestamp)));
                                        }
                                    });
                            }
                        }

                        // no bans found
                        if all_bans.is_empty() {

                            // resolve bot
                            let bot_id: UserId = ConfigDB::get_instance()
                                .get("bot_id").await.unwrap().into();
                            let bot = message.get_resolver().resolve_user(bot_id).await.unwrap();

                            // create embed
                            let embed = message.get_log_builder()
                                .title(format!("{} has not been banned before.", name))
                                .target(&bot)
                                .no_thumbnail()
                                .build().await;
                            let _ = message.reply(embed).await;
                            return;
                        }

                        // bans found
                        else {

                            // create embed
                            let mut builder = message.get_log_builder()
                                .no_thumbnail()
                                .title(format!("{}'s Bans", name));

                            // add target info if available
                            if let Some(ref target) = target {
                                builder = builder.target(target);
                            }

                            // add reasons if available
                            let reasons = all_bans.iter()
                                .map(|ban| {
                                    match ban.1 {
                                        Some(timestamp) => format!("<t:{}> `>` {}", timestamp, ban.0),
                                        None => format!("No timestamp available `>` {}", ban.0),
                                    }
                                })
                                .collect::<Vec<_>>();
                            builder = builder.description(reasons.join("\n"));

                            // add bans to embed
                            let embed = builder.build().await;
                            let _ = message.reply(embed).await;
                        }
                    }
                }
            }
        )
    }
}


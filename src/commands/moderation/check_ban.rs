
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct CheckBanCommand;

impl Command for CheckBanCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "check-ban".to_string(),
            "checkban".to_string(),
            "bans".to_string(),
        ])
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
                let bans = BansDB::get_instance()
                    .get_all(target_id).await;

                if let Ok(bans) = bans {

                    // no bans found
                    if bans.is_empty() {

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
                            .no_thumbnail();

                        if let Some(ref target) = target {
                            builder = builder
                                .target(target)
                                .title(format!("{}'s Bans", name));
                        } else {
                            builder = builder.title(format!("{}' Bans", name));
                        }

                        // add bans to embed
                        for ban in bans.into_iter() {
                            builder = builder.mod_log(&ban);
                        }
                        let embed = builder.build().await;
                        let _ = message.reply(embed).await;
                    }

                }

            }
        )
    }

}



use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct TicketStatsCommand;

impl Command for TicketStatsCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "ticket-stats".to_string(),
        ])
            .add_optional("user")
    }

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;

                let mut target_ids = message.get_mentions().await.into_iter()
                    .map(|user_id| user_id.to_string())
                    .collect::<Vec<_>>();
                if target_ids.is_empty() {
                    target_ids = TicketReviewsDB::get_instance()
                        .get_keys().await;
                }

                // resolve bot
                let bot_id: UserId = ConfigDB::get_instance()
                    .get("bot_id").await.unwrap().into();
                let bot = message.get_resolver().resolve_user(bot_id).await.unwrap();

                let mut builder = message.get_log_builder()
                    .title("Ticket Statistics")
                    .target(&bot);

                // collect statistics for each target
                for target_id in target_ids {

                    let user_id = UserId::from_str(&target_id).unwrap();
                    let user = message.get_resolver().resolve_user(user_id).await.unwrap();
                    let name = message.get_resolver().resolve_name(&user);

                    let reviews = TicketReviewsDB::get_instance()
                        .get_all(&target_id).await.unwrap();

                    if reviews.is_empty() {
                        builder = builder
                            .arbitrary_block(name, "No reviews found.".to_string());
                        continue;
                    }

                    let approved = reviews.iter().filter(|review| review.approved).count();
                    builder = builder
                        .arbitrary_block(name, format!(
                            "{}% ({}/{})", approved * 100 / reviews.len(), approved, reviews.len()
                        ));
                }

                let embed = builder.build().await;
                let _ = message.reply(embed.to_message()).await;
            }
        )
    }

}


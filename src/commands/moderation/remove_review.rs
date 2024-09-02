
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{CommandType, Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemoveReviewCommand;

impl Command for RemoveReviewCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["remove-review".to_string(), "remove-ticket-review".to_string()]
        )
            .add_required("database ID")
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
                let review_id = params.number.unwrap();

                let review = TicketReviewsDB::get_instance()
                    .query("", &format!("OR id = {}", review_id)).await;
                if review.is_err() || review.clone().unwrap().is_empty() {
                    message.reply_failure("Review not found.").await;
                    return;
                }
                let review = &review.unwrap()[0];

                // remove review
                TicketReviewsDB::get_instance().delete_by_id(review_id).await;

                // resolve target
                let user_id = UserId::from(review.key.parse::<u64>().unwrap());
                let target = message.get_resolver().resolve_user(user_id).await.unwrap();

                // log to mod logs
                let log_message = message.get_log_builder()
                    .title("[REMOVE REVIEW]")
                    .description(&format!("Removed review with **ID {}**", review_id))
                    .target(&target)
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .timestamp()
                    .build().await;
                let modlogs: ChannelId = ConfigDB::get_instance()
                    .get("channel_modlogs").await.unwrap().into();
                let _ = modlogs.send_message(message, log_message.to_message()).await;

                message.reply_success().await;
            }
        )
    }

}



use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct TicketReviewsCommand;

impl Command for TicketReviewsCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "ticket-reviews".to_string(),
            "reviews".to_string()
        ])
            .add_required("user")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                let reviews = TicketReviewsDB::get_instance()
                    .get_all(&target.id.to_string()).await;

                if let Ok(reviews) = reviews {

                    // split reviews into chunks of 25
                    let reviews = reviews.chunks(25).collect::<Vec<_>>();

                    // create embed
                    let name = message.get_resolver().resolve_name(target);

                    match reviews.len() {
                        0 => {
                            let embed = message.get_log_builder()
                                .target(target)
                                .no_thumbnail()
                                .title(format!("{}'s Reviews", name))
                                .description("No reviews found.")
                                .build().await;
                            let _ = message.reply(embed).await;
                        },
                        _ => {
                            for (index, chunk) in reviews.iter().enumerate() {
                                let mut builder = message.get_log_builder()
                                    .target(target)
                                    .no_thumbnail()
                                    .title(format!("{}'s Reviews", name));

                                // add fields for each review
                                for entry in chunk.iter() {

                                    builder = builder.arbitrary_block(
                                        format!("**Databse ID**: {}", entry.id),
                                        format!("<@{}> `>` {}\n**Notes**: {}",
                                            target.id,
                                            match entry.approved { true => "Approved", false => "Denied" },
                                            entry.notes));
                                }
                                let embed = builder.build().await
                                    .footer(CreateEmbedFooter::new(
                                        format!("Page {} of {}", index + 1, reviews.len())));

                                let _ = message.reply(embed).await;
                            }
                        }
                    };
                }
            }
        )
    }

}



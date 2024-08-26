
use serenity::all::ReactionType;
use nonempty::{NonEmpty, nonempty};
use futures::StreamExt;

use std::sync::Arc;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct PollCommand;

impl Command for PollCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "poll".to_string(),
        ])
            .add_constant("-title", true)
            .add_constant("-opts", false)
            .add_required("+option1")
            .add_optional("+option2 .. +option9")
            .example("-title What is the best color? -opts +Reddish Blue +Blue +Green")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;

                if !message.has_parameter("title") {
                    self.invalid_usage(params).await;
                    return;
                }

                if !message.has_parameter("opts") {
                    self.invalid_usage(params).await;
                    return;
                }

                let title = message.get_parameter("title");
                let payload = message.get_parameter("opts");
                let options = payload
                    .split("+")
                    .map(|option| option.trim())
                    .filter(|option| !option.is_empty())
                    .collect::<Vec<_>>();

                if options.len() <= 2 {
                    self.invalid_usage(params).await;
                    return;
                }

                if options.len() >= 10 {
                    self.invalid_usage(params).await;
                    return;
                }

                let emojis: Vec<_> = vec!["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣"]
                    .into_iter()
                    .take(options.len())
                    .map(|emoji| emoji.to_string())
                    .collect();
                let description = options.into_iter()
                    .enumerate()
                    .map(|(i, option)| format!("{} - {}", emojis[i], option))
                    .collect::<Vec<_>>()
                    .join("\n");

                let embed = message.get_log_builder()
                    .no_thumbnail()
                    .title(title)
                    .description(description)
                    .timestamp()
                    .build().await;

                if let Ok(sent_message) = message.reply(embed).await {
                    for emoji in emojis {
                        let _ = sent_message.react(&message, ReactionType::Unicode(emoji)).await;
                    }
                }
            }
        )
    }

}


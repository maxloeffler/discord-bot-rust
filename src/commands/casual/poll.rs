
use serenity::all::ReactionType;
use nonempty::{NonEmpty, nonempty};
use futures::StreamExt;

use std::sync::Arc;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct PollCommand;

impl Command for PollCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["poll".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;

                if !message.has_parameter("title") {
                    let _ = message.reply_failure("Please provide a title (`-title`)!").await;
                    return;
                }

                if !message.has_parameter("opts") {
                    let _ = message.reply_failure("Please provide options (`-opts`)!").await;
                    return;
                }

                let title = message.get_parameter("title");
                let payload = message.get_parameter("opts");
                let options = payload
                    .split("+")
                    .map(|option| option.trim())
                    .filter(|option| !option.is_empty())
                    .collect::<Vec<_>>();

                println!("{}", payload);
                println!("{:?}", options);

                if options.len() <= 2 {
                    let _ = message.reply_failure("Please provide at least 2 options!").await;
                    return;
                }

                if options.len() >= 10 {
                    let _ = message.reply_failure("Please provide at most 9 options!").await;
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
                println!("{}", description);

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



use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct SuggestCommand;

impl Command for SuggestCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "suggest".to_string(),
        ])
            .add_required("message")
            .example("Add unicorns to planet earth!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let content = message.payload(None, None);

                if content.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                // create embed
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title("Pending Suggestion")
                        .description(&content)
                        .footer(CreateEmbedFooter::new(
                            format!("Suggested by {}", message.resolve_name())
                        ))
                }).await;
                let reactions = vec![ReactionType::Unicode("✅".to_string()),
                                     ReactionType::Unicode("❌".to_string())];
                let suggestion = embed.to_message().reactions(reactions);

                // send to suggestions channel
                let channel: ChannelId = ConfigDB::get_instance().lock().await
                    .get("channel_suggestions").await.unwrap().into();
                let _ = channel.send_message(&message, suggestion).await;

                let _ = message.reply_success().await;
            }
        )
    }
}




use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::message_manager::MessageManager;
use crate::utility::mixed::BoxedFuture;
use crate::utility::traits::{Singleton};
use crate::databases::*;


pub struct WarnCommand;

impl Command for WarnCommand {

    fn permission(&self, message: MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["warn".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let message = params.message;
                let mentions = message.get_mentions().await;

                if mentions.len() == 0 {
                    message.reply_failure("You need to mention someone to warn them.").await;
                    return;
                }

                let target = mentions[1].clone();
                let resolver = message.get_resolver();
                if resolver.is_trial(target.clone()).await {
                    message.reply_failure("You can't warn a moderator.").await;
                    return;
                }

                let mut reason = message.payload_without_mentions(None, None).await;
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }

                let log = ModLog {
                    member_id: target.clone().id.to_string(),
                    staff_id: message.get_author().id.to_string(),
                    reason: reason.clone(),
                };
                WarningsDB::get_instance().lock().await
                    .append(&target.id.to_string(), &log.into()).await;
                let embed = MessageManager::create_embed(|embed|
                    embed
                        .title(&format!("Warning"))
                        .description(&format!("<@{}> has been warned for `>` {}", target.id, reason))
                        .color(0xff0000)
                ).await;
                message.reply(embed).await;
            }
        )
    }

}



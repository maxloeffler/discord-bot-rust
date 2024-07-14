
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::message_manager::MessageManager;
use crate::utility::mixed::BoxedFuture;


pub struct WarnCommand;

impl Command for WarnCommand {

    fn permission(&self, message: MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial(None).await
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
                if message.is_trial(Some(target.clone())).await {
                    message.reply_failure("You can't warn a moderator.").await;
                    return;
                }

                message.reply(format!("{} has been warned.", target.name).as_str()).await;
            }
        )
    }

}



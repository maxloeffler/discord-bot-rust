
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct AfkCommand;

impl Command for AfkCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["afk".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let content = &message.payload(None, None);

                if content.len() >= 154 {
                    message.reply_failure("Your afk message is too long (>154)!").await;
                    return;
                }

                AfkDB::get_instance().lock().await
                    .set(&message.get_author().id.to_string(), content).await;

                message.reply_success().await;
            }
        )
    }

}


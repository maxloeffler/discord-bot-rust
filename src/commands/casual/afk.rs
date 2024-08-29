
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct AfkCommand;

impl Command for AfkCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["afk".to_string()]
        )
            .add_optional("message (max 154 characters)")
            .example("I am going afk now :)")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let content = &message.payload(None, None);

                if content.len() >= 154 {
                    self.invalid_usage(params).await;
                    return;
                }

                AfkDB::get_instance()
                    .set(&message.get_author().id.to_string(), content).await;

                message.reply_success().await;
            }
        )
    }

}


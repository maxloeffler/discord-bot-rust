
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{CommandType, Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemoveReminderCommand;

impl Command for RemoveReminderCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["remove-reminder".to_string()]
        )
            .add_required("database ID")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let id = message.payload(None, None);

                if id.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }
                let id = id.parse::<i64>();

                if id.is_err() {
                    self.invalid_usage(params).await;
                    return;
                }
                let id = id.unwrap();

                RemindersDB::get_instance().delete_by_id(id).await;

                message.reply_success().await;
            }
        )
    }
}



use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{CommandType, Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemoveNoteCommand;

impl Command for RemoveNoteCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["remove-note".to_string(),"removenote".to_string()]
        )
            .add_required("label")
            .example("Deprecated Note")
    }

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;

                let label = Note::escape(message.payload(None, None));
                if label.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }
                NotesDB::get_instance().delete(&label).await;

                message.reply_success().await;
            }
        )
    }
}


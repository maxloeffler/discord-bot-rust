
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemoveNoteCommand;

impl Command for RemoveNoteCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "remove-note".to_string(),
            "removenote".to_string(),
        ])
            .add_required("label")
            .example("Deprecated Note")
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

                NotesDB::get_instance().lock().await
                    .delete(&label).await;

                message.reply_success().await;
            }
        )
    }
}


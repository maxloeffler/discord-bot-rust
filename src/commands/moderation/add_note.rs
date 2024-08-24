
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct AddNoteCommand;

impl Command for AddNoteCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "add-note".to_string(),
            "addnote".to_string(),
        ])
            .add_required("-label")
            .add_required("label")
            .add_required("-content")
            .add_required("content")
            .example("add-note -label Edate -content Edating is a great falacy!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;

                if !message.has_parameter("content") || !message.has_parameter("label") {
                    self.invalid_usage(params).await;
                    return;
                }

                let label = Note::escape(message.get_parameter("label"));
                let content = message.get_parameter("content");

                let note = Note::new(content);
                NotesDB::get_instance().lock().await
                    .set(&label, &note.into()).await;

                message.reply_success().await;
            }
        )
    }
}


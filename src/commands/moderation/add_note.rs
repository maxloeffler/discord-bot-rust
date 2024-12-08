
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{CommandType, Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct AddNoteCommand;

impl Command for AddNoteCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["add-note".to_string(), "addnote".to_string()]
        )
            .add_constant(vec!["-label", "-content"], true)
            .example("-label Edate -content Edating is a great fallacy!")
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

                if !message.has_parameter("content") || !message.has_parameter("label") {
                    self.invalid_usage(params).await;
                    return;
                }

                let label = Note::escape(message.get_parameter("label"));
                let content = message.get_parameter("content");

                let note = Note::new(content);
                NotesDB::get_instance().set(&label, &note.into()).await;

                message.reply_success().await;
            }
        )
    }
}


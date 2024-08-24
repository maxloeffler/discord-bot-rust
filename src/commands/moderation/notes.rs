
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct NotesCommand;

impl Command for NotesCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "notes".to_string(),
            "cc".to_string(),
        ])
            .add_optional("label")
            .new_usage()
            .add_required("-list")
            .example("notes edate")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let label = Note::escape(message.payload(None, None));

                // list all notes
                if message.has_parameter("list") || label.is_empty() {

                    // get all notes
                    let notes = NotesDB::get_instance().lock().await
                        .get_keys().await
                        .into_iter()
                        .map(|key| format!("`{}`", Note::deescape(key)))
                        .collect::<Vec<String>>()
                        .join("\n");

                    // create embed
                    let embed = MessageManager::create_embed(|embed| {
                        embed
                            .title("List of all Notes")
                            .description(&notes)
                    }).await;
                    let _ = message.reply(embed).await;

                // display single note
                } else {

                    let note_keys = NotesDB::get_instance().lock().await
                        .get_keys().await;
                    let note_keys = note_keys
                        .iter()
                        .map(|key| key.as_str())
                        .collect::<Vec<&str>>();
                    let notes = NotesDB::get_instance().lock().await
                        .get_multiple(note_keys).await.unwrap();

                    // match note
                    let triggerables = notes.iter()
                        .map(|note| note as &dyn Triggerable)
                        .collect::<Vec<_>>();
                    let index = match_triggerables(message, &label, triggerables).await;

                    if let Ok(index) = index {
                        let note = &notes[index];

                        // create embed
                        let embed = MessageManager::create_embed(|embed| {
                            embed
                            .title(&Note::deescape(note.key.clone()))
                            .description(&note.content)
                        }).await;

                        let _ = message.reply(embed).await;
                    }
                }
            }
        )
    }
}


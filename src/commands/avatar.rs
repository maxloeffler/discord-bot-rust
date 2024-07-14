
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::Command;
use crate::utility::mixed::BoxedFuture;
use crate::utility::message_manager::MessageManager;


pub struct AvatarCommand;

impl Command for AvatarCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["avatar".to_string(), "av".to_string()]
    }

    fn run(&self, message: MessageManager) -> BoxedFuture<'_> {
        Box::pin(
            async move {
                let author = message.get_author();
                let name = author.clone().global_name.unwrap_or(author.clone().name);
                let face = author.clone().face();
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title(&format!("{}'s avatar", name))
                        .image(face.clone())
                }).await;
                match embed {
                    Ok(embed) => message.reply(embed).await,
                    Err(_) => message.reply(face).await
                }
            }
        )
    }

}



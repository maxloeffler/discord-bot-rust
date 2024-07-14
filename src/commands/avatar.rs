
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::message_manager::MessageManager;
use crate::utility::mixed::BoxedFuture;


pub struct AvatarCommand;

impl Command for AvatarCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["avatar".to_string(), "av".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                if let Some(author) = params.target {
                    let name = author.clone().global_name.unwrap_or(author.clone().name);
                    let face = author.clone().face();
                    let embed = MessageManager::create_embed(|embed| {
                        embed
                            .title(&format!("{}'s avatar", name))
                            .image(face.clone())
                    }).await;
                    match embed {
                        Ok(embed) => params.message.reply(embed).await,
                        Err(_)    => params.message.reply(face).await
                    }
                }
            }
        )
    }

}



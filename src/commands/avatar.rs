
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
                let target = params.target.unwrap().clone();
                let name = params.message.get_resolver()
                    .resolve_name(target.clone());
                let face = target.clone().face();
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title(&format!("{}'s avatar", name))
                        .image(face.clone())
                }).await;
                params.message.reply(embed).await
            }
        )
    }

}



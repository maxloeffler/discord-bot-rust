
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;


pub struct AvatarCommand;

impl Command for AvatarCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["avatar".to_string(), "av".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let message = params.message;
                let target = params.target.unwrap();

                let embed = message.get_log_builder()
                    .title(&format!("{}'s avatar", message.get_resolver().resolve_name(&target)))
                    .image(target.face())
                    .target(&target)
                    .no_thumbnail()
                    .build().await;
                message.reply(embed).await
            }
        )
    }

}



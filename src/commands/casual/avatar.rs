
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;


pub struct AvatarCommand;

impl Command for AvatarCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["av".to_string(), "avatar".to_string()]
        )
            .add_required("user")
            .example("@Poggy")
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
                let _ = message.reply(embed).await;
            }
        )
    }

}



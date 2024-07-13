
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{BoxedFuture, Command};
use crate::utility::message_manager::MessageManager;


pub struct AvatarCommand;

impl Command for AvatarCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["avatar".to_string(), "av".to_string()]
    }

    fn run(&self, message: MessageManager) -> BoxedFuture<'_> {
        Box::pin(
            async move {
                let avatar = message.get_author().face();
                message.reply(&avatar).await;
            }
        )
    }

}



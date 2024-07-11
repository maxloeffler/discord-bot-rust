
use crate::commands::command::{BoxedFuture, Command};
use crate::utility::message_manager::MessageManager;


pub struct AvatarCommand;

impl Command for AvatarCommand {

    fn get_names(&self) -> Vec<String> {
        vec!["avatar".to_string(), "av".to_string()]
    }

    fn run(&self, message: MessageManager) -> BoxedFuture<'_> {
        Box::pin(
            async move {
                let avatar = message.get_author().avatar_url().unwrap();
                message.reply(&avatar).await;
            }
        )
    }

}



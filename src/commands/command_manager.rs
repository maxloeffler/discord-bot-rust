
use tokio::sync::Mutex;

use std::sync::Arc;

use crate::utility::database::Database;
use crate::utility::message_manager::MessageManager;
use crate::commands::command::Command;
use crate::commands::avatar::AvatarCommand;


pub struct CommandManager {
    config: Arc<Mutex<Database>>,
    message: MessageManager,
    commands: Vec<Box<dyn Command>>,
}

impl CommandManager {

    pub async fn new(config: Arc<Mutex<Database>>, message: MessageManager) -> CommandManager {
        let manager = CommandManager {
            config,
            message,
            commands: vec![Box::new(AvatarCommand {})],
        };
        manager
    }

    pub fn match_command(&self) -> Option<&Box<dyn Command>> {
        if self.message.is_command() {
            let trigger = self.message.get_command().unwrap();
            for command in self.commands.iter() {
                if command.woke_by(trigger.to_string()) {
                    return Some(command);
                }
            }
        }
        None
    }

    pub async fn execute(&self) {
        let command = self.match_command();
        if command.is_some() {
            let command = command.unwrap();
            if command.permission() {
                command.run(self.message.clone()).await;
            }
        }
    }

}

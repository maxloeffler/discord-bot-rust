
use crate::utility::message_manager::MessageManager;
use crate::commands::*;


pub struct CommandManager {
    commands: Vec<Box<dyn Command>>,
}

impl CommandManager {

    pub async fn new() -> CommandManager {
        let manager = CommandManager {
            commands: vec![
                Box::new(AvatarCommand {}),
            ],
        };
        manager
    }

    fn match_command(&self, trigger: String) -> Option<&Box<dyn Command>> {
        for command in self.commands.iter() {
            if command.is_triggered_by(trigger.clone()) {
                return Some(command);
            }
        }
        None
    }

    // note: only execute this method, when message.is_command() is true
    pub async fn execute(&self, message: MessageManager) {
        let trigger = message.get_command().unwrap();
        let matched_command = self.match_command(trigger);
        if matched_command.is_some() {
            let command = matched_command.unwrap();
            if command.permission() {
                command.run(message).await;
            }
        }
    }

}

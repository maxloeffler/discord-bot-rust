
use crate::commands::command::{CommandParams, MatchType};
use crate::utility::*;
use crate::commands::*;


#[cfg(feature = "commands")]
pub struct CommandManager {
    commands: Vec<Box<dyn Command>>,
}

#[cfg(feature = "commands")]
impl CommandManager {

    pub fn new() -> CommandManager {
        let manager = CommandManager {
            commands: vec![
                Box::new( UserDecorator{ command: Box::new(AvatarCommand{}) }),
                Box::new( UserDecorator{ command: Box::new(InfoCommand{}) }),
                Box::new( WarnCommand{} ),
            ],
        };
        manager
    }

    async fn run_command(&self, command: &Box<dyn Command>, message: &MessageManager) {
        if command.permission(message).await {
            message.delete().await;
            command.run(CommandParams::new(message.clone(), None)).await;
        }
    }

    // note: only execute this method, when message.is_command() is true
    pub async fn execute(&self, message: &MessageManager) {
        for command in self.commands.iter() {
            match command.is_triggered_by(message) {
                MatchType::Exact => self.run_command(command, message).await,
                MatchType::Fuzzy(closest_match) => {

                    // prepare correction message
                    let correction = format!("{}{} {}",
                        message.get_prefix().unwrap(),
                        closest_match,
                        message.payload(None, None));
                    let embed = MessageManager::create_embed(|embed| {
                        embed.title("Did you mean ...").description(&correction)
                    }).await;

                    // send correction message
                    message.create_choice_interaction(
                        embed,
                        Box::pin( async move { self.run_command(command, message).await } ),
                        Box::pin( async move {} )
                    ).await;
                    return;
                },
                MatchType::None => continue,
            };
        }
    }

}

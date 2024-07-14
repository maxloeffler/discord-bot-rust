
use crate::utility::message_manager::MessageManager;
use crate::commands::*;
use crate::commands::command::MatchType;


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

    pub async fn run_command(&self, command: &Box<dyn Command>, message: MessageManager) {
        if command.permission() {
            command.run(message).await;
        }
    }

    // note: only execute this method, when message.is_command() is true
    pub async fn execute(&self, message: MessageManager) {
        for command in self.commands.iter() {
            match command.is_triggered_by(message.clone()) {
                MatchType::Exact => self.run_command(command, message.clone()).await,
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
                    if let Ok(embed) = embed {
                        message.clone().create_choice_interaction(
                            embed,
                            Box::pin( async move { self.run_command(command, message.clone()).await } ),
                            Box::pin( async move {} )
                        ).await;
                        return;
                    }
                },
                MatchType::None => continue,
            };
        }
    }

}

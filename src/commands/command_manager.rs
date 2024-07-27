
use crate::commands::command::{CommandParams, MatchType};
use crate::utility::*;
use crate::commands::*;
use crate::databases::*;


#[cfg(feature = "commands")]
pub struct CommandManager {
    commands: Vec<Box<dyn Command>>,
}

#[cfg(feature = "commands")]
impl CommandManager {

    pub fn new() -> CommandManager {
        let mut commands: Vec<Box<dyn Command>> = vec![
            // casual commands
            Box::new( UserDecorator{ command: Box::new(AvatarCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(InfoCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(NicknameCommand{}) }),
            Box::new( VerifyCommand{} ),
            Box::new( AboutCommand{} ),
            Box::new( ServerInfoCommand{} ),
            Box::new( AfkCommand{} ),
            Box::new( PollCommand{} ),
            // moderation commands
            Box::new( WarnCommand{} ),
            Box::new( UserDecorator{ command: Box::new(WarningsCommand{}) }),
            Box::new( PurgeCommand{} ),
            Box::new( SlowmodeCommand{} ),
            Box::new( MuteCommand{} ),
            Box::new( UserDecorator{ command: Box::new(UnmuteCommand{}) }),
        ];
        #[cfg(feature = "tickets")]
        let ticket_commands: Vec<Box<dyn Command>> = vec![
            // ticket command
            Box::new( UserDecorator{ command: Box::new(OpenTicketCommand{}) }),
            Box::new( CloseTicketCommand{} ),
            Box::new( ClaimTicketCommand{} ),
            Box::new( UnclaimTicketCommand{} ),
            Box::new( UserDecorator{ command: Box::new(AddMemberToTicketCommand) }),
            Box::new( UserDecorator{ command: Box::new(RemoveMemberFromTicketCommand) }),
        ];
        #[cfg(feature = "tickets")]
        commands.extend(ticket_commands);
        let manager = CommandManager { commands };
        manager
    }

    async fn run_command(&self, command: &Box<dyn Command>, message: &MessageManager) {

        if command.permission(message).await {

            // execute command
            message.delete().await;
            command.run(CommandParams::new(message.clone(), None)).await;

            // increment executed commands
            let executed_commands = ConfigDB::get_instance().lock().await.
                get("executed_commands").await.unwrap().to_string().parse::<i64>().unwrap() + 1;
            ConfigDB::get_instance().lock().await.
                set("executed_commands", &executed_commands.to_string()).await;

        } else {
            message.reply_failure("You do not have permission to use this command").await;
        }
    }

    // note: only execute this method, when message.is_command() is true
    pub async fn execute(&self, message: &MessageManager) {

        // initialize search
        let mut fuzzy_matches = Vec::new();
        let mut exact_match = false;

        // search for command
        for command in self.commands.iter() {
            match command.is_triggered_by(message) {
                MatchType::Exact => {
                    self.run_command(command, message).await;
                    exact_match = true;
                    break;
                },
                MatchType::Fuzzy(closest_match) => fuzzy_matches.push((command, closest_match)),
                MatchType::None => continue,
            };
        }
        if !exact_match {
            for (command, closest_match) in fuzzy_matches {

                // create correction message
                let correction = format!("{}{} {}",
                    message.get_prefix().unwrap(),
                    closest_match,
                    message.payload(None, None));

                // create embed
                let embed = MessageManager::create_embed(|embed| {
                    embed.title("Did you mean ...").description(&correction)
                }).await;

                // create choice interaction
                message.create_choice_interaction(
                    embed,
                        Box::pin( async move { self.run_command(command, message).await } ),
                        Box::pin( async move {} )
                ).await;
            }
        }
    }

}


use serenity::builder::CreateButton;
use serenity::all::ButtonStyle;
use serenity::all::UserId;
use strum::IntoEnumIterator;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::str::FromStr;

use crate::commands::command::{CommandType, CommandParams};
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
            Box::new( AddEmojiCommand{} ),
            Box::new( TimeDecorator{ command: Box::new(ScheduleCommand{}) }),
            Box::new( SuggestCommand{} ),
            Box::new( RemindCommand{} ),
            Box::new( NumberDecorator{ command: Box::new(RemoveReminderCommand{}) }),
            Box::new( TweetCommand{} ),
            Box::new( DeadchatCommand{} ),
            // games commands
            Box::new( EightBallCommand{} ),
            // moderation commands
            Box::new( WarnCommand{} ),
            Box::new( UserDecorator{ command: Box::new(WarningsCommand{}) }),
            Box::new( NumberDecorator{ command: Box::new(PurgeCommand{}) }),
            Box::new( SlowmodeCommand{} ),
            Box::new( MuteCommand{} ),
            Box::new( UserDecorator{ command: Box::new(UnmuteCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(RemoveAfkCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(RoleCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(LockCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(UnlockCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(FlagCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(UnflagCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(FlagsCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(BanCommand{}) }),
            Box::new( CheckBanCommand{} ),
            Box::new( UnbanCommand{} ),
            Box::new( NumberDecorator{ command: Box::new(RemoveWarningCommand{}) }),
            Box::new( ReviewCommand{} ),
            Box::new( NumberDecorator{ command: Box::new(RemoveReviewCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(ModStatsCommand{}) }),
            Box::new( LockdownCommand{} ),
            Box::new( NotesCommand{} ),
            Box::new( AddNoteCommand{} ),
            Box::new( RemoveNoteCommand{} ),
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
            Box::new( TicketSelectorCommand{} ),
            Box::new( UserDecorator{ command: Box::new(TicketReviewsCommand{}) }),
            Box::new( TicketStatsCommand{} ),
            Box::new( ResetTicketsCommand{} ),
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
            command.run(CommandParams::new(message.clone())).await;

            // increment executed commands
            let executed_commands = ConfigDB::get_instance()
                .get("executed_commands").await.unwrap().to_string().parse::<i64>().unwrap() + 1;
            ConfigDB::get_instance()
                .set("executed_commands", &executed_commands.to_string()).await;

        } else {
            message.reply_failure("You do not have permission to use this command").await;
        }
    }


    // note: only execute this method, when message.is_command() is true
    pub async fn execute(&self, message: &MessageManager) {

        // special case: help (needs more permissions)
        if message.get_command().unwrap() == "help" {
            self.display_help(message).await;
            return;
        }

        // match command
        let triggerables = self.commands.iter()
            .map(|command| command as &dyn Triggerable)
            .collect::<Vec<_>>();
        let index = match_triggerables(message, &message.get_command().unwrap(), triggerables).await;

        // execute command if found
        if let Ok(index) = index {
            let command = &self.commands[index];
            self.run_command(command, message).await;
        }
    }

    async fn display_help(&self, message: &MessageManager) {

        // delete message
        message.delete().await;

        // resolve bot user
        let bot_id: UserId = ConfigDB::get_instance()
            .get("bot_id").await.unwrap().into();
        let bot = message.get_resolver().resolve_user(bot_id).await.unwrap();

        // display categories
        let payload = message.payload(None, None);
        if payload.is_empty() {

            let categories = CommandType::iter()
                .map(|category| format!("`{}`", category.to_string()))
                .collect::<Vec<_>>()
                .join(", ");
            let description = format!("Commands are categorized into groups which are listed below. If you cannot find the command you are searching for or if you need help with anything else, you can make a ticket through our tickets channel.\n\n**Categories**\n{categories}");

            // create embed
            let embed = message.get_log_builder()
                .target(&bot)
                .no_thumbnail()
                .title("Available Commands")
                .description(&description)
                .build().await;
            let _ = message.reply(embed.to_message()).await;
        }

        // display help for a specific command
        else {

            // find command
            let trigger = &payload.split_whitespace().next().unwrap().to_string();

            for command_type in CommandType::iter() {
                if command_type.to_string().to_lowercase() == *trigger.to_lowercase() {

                    // filter commands
                    let mut commands = Vec::new();
                    for command in self.commands.iter() {
                        if command.define_usage().command_type == command_type {
                            commands.push(command);
                        }
                    }

                    let mut command_strings = commands.iter()
                        .map(|command| format!("`{}{}`", message.get_prefix().unwrap(), command.trigger()))
                        .collect::<Vec<_>>();
                    command_strings.sort();

                    // create embed
                    let embed = message.get_log_builder()
                        .target(&bot)
                        .no_thumbnail()
                        .title(format!("{} Commands", command_type.to_string()))
                        .description(&command_strings.join(", "))
                        .build().await;
                    let _ = message.reply(embed.to_message()).await;
                    return;
                }
            }

            // match command
            let triggerables = self.commands.iter()
                .map(|command| command as &dyn Triggerable)
                .collect::<Vec<_>>();
            let index = match_triggerables(message, &trigger, triggerables).await;

            // display usage
            if let Ok(index) = index {
                let command = &self.commands[index];
                let params = CommandParams::new(message.clone());
                command.display_usage(params, "Command Description".to_string()).await;
            }
        }
    }
}

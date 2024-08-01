
use serenity::builder::CreateButton;
use serenity::all::ButtonStyle;
use serenity::all::UserId;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::str::FromStr;

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
            Box::new( UserDecorator{ command: Box::new(RemoveAfkCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(RoleCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(LockCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(UnlockCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(FlagCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(UnflagCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(FlagsCommand{}) }),
            Box::new( UserDecorator{ command: Box::new(BanCommand{}) }),
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

    async fn match_command<'a>(&'a self,
            message: &'a MessageManager,
            match_callback: impl Fn(&'a Box<dyn Command>) -> BoxedFuture<'a, ()> + Send + Copy
    ) {

        // initialize search
        let mut fuzzy_matches = Vec::new();
        let mut exact_match = false;

        for command in self.commands.iter() {
            match command.is_triggered_by(message) {
                MatchType::Exact => {
                    match_callback(command).await;
                    exact_match = true;
                    break;
                },
                MatchType::Fuzzy(closest_match) => fuzzy_matches.push((command, closest_match)),
                MatchType::None => continue,
            };
        }
        if !exact_match {

            // create buttons
            let buttons = fuzzy_matches.iter().enumerate()
                .map(|(i, (_, closest_match))| {
                    CreateButton::new(i.to_string())
                        .label(closest_match)
                        .style(ButtonStyle::Secondary)
                }).collect::<Vec<_>>();

            // create embed
            let cmd = message.get_command().unwrap();
            let embed = MessageManager::create_embed(|embed| {
                embed
                    .title("Did you mean ...")
                    .description(&format!("`{}{}` {}\n`{}`",
                            message.get_prefix().unwrap(),
                            cmd,
                            message.payload(None, None),
                            "^".repeat(1 + cmd.len())))
            }).await;

            // create interaction
            let interaction_helper = message.get_interaction_helper();
            let pressed = interaction_helper.create_buttons(
                embed,
                buttons
            ).await;

            // execute callback
            if let Some(pressed) = pressed {
                let index = pressed.parse::<usize>().unwrap();
                let (command, _) = fuzzy_matches.get(index).unwrap();
                match_callback(command).await;
            }
        }
    }

    // note: only execute this method, when message.is_command() is true
    pub async fn execute(&self, message: &MessageManager) {

        // special case: help (needs more permissions)
        if message.get_command().unwrap() == "help" {
            self.display_help(message).await;
            return;
        }

        self.match_command(message, |command: &Box<dyn Command>| {
            Box::pin(async move {
                self.run_command(command, message).await
            })}).await;
    }

    async fn display_help(&self, message: &MessageManager) {

        // delete message
        message.delete().await;

        // resolve bot user
        let bot_id = ConfigDB::get_instance().lock().await
            .get("bot_id").await.unwrap().to_string();
        let bot = message.get_resolver()
            .resolve_user(UserId::from_str(bot_id.as_str()).unwrap()).await.unwrap();

        // display all available commands
        let payload = message.payload(None, None);
        if payload.is_empty() {

            // filter commands
            let mut allowed_commands = Vec::new();
            for command in self.commands.iter() {
                if command.permission(message).await {
                    allowed_commands.push(command);
                }
            }

            // collect commands
            let prefix = message.get_prefix().unwrap();
            let description = allowed_commands.into_iter()
                .map(|command| {
                    format!("`{}{}`", prefix, command.define_usage().triggers[0].clone())
                })
                .collect::<Vec<_>>()
                .join("\n");

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
            let trigger = payload.split_whitespace().next().unwrap();
            let message = &message.spoof(
                format!("{}{}", message.get_prefix().unwrap(), trigger)).await;

            self.match_command(message, |command: &Box<dyn Command>| {
                Box::pin(async move {
                    let params = CommandParams::new(message.clone(), None);
                    command.display_usage(params, "Command Description".to_string()).await;
                })}).await;

        }
    }

}

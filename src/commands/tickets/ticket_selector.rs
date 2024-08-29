
use serenity::model::channel::ReactionType;
use serenity::builder::{CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};
use serenity::model::id::UserId;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct TicketSelectorCommand;

impl Command for TicketSelectorCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_admin().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "select-ticket".to_string(),
            "ticket-selector".to_string(),
        ])
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;
                let bot_id: UserId = ConfigDB::get_instance()
                    .get("bot_id").await.unwrap().into();
                let bot = message.get_resolver().resolve_user(bot_id).await.unwrap();

                let options = vec![
                    CreateSelectMenuOption::new("Staff Report", TicketType::StaffReport).emoji(ReactionType::Unicode("📁".to_string())),
                    CreateSelectMenuOption::new("User Report",  TicketType::UserReport).emoji(ReactionType::Unicode("💼".to_string())),
                    CreateSelectMenuOption::new("Bug Report",   TicketType::BugReport).emoji(ReactionType::Unicode("📔".to_string())),
                    CreateSelectMenuOption::new("Question",     TicketType::Question).emoji(ReactionType::Unicode("🤔".to_string())),
                ];

                let selector = message.get_log_builder()
                    .target(&bot)
                    .title("Kalopsia's Support Tickets")
                    .arbitrary_block("📁 Staff Report", "> Report a member of the staff team to a Head-Moderator or Administrator. Be sure to have evidence ready.")
                    .arbitrary_block("💼 User Report",  "> Report a user of the server to the staff team. Be sure to have evidence ready.")
                    .arbitrary_block("📔 Bug Report",   "> Help us improve the server by reporting bugs or issues you encounter.")
                    .arbitrary_block("🤔 Question",     "> Do you have any questions about the server? Ask them here.")
                    .build().await.to_message()
                    .select_menu(
                        CreateSelectMenu::new("ticket_selector", CreateSelectMenuKind::String {
                            options
                        })
                        .placeholder("Select a ticket")
                    );

                let selector = message.reply(selector).await;
                if let Ok(selector) = selector {
                    spawn(hook_ticket_selector(message.get_resolver().clone(), selector)).await;
                }
            }
        )
    }

}



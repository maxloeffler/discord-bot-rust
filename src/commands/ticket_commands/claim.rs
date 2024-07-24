
use serenity::builder::CreateMessage;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct ClaimTicketCommand;

impl Command for ClaimTicketCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["claim".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;
                let staff = message.get_author().id;
                let ticket = TicketHandler::get_instance().lock().await
                    .get_ticket(&message.get_channel()).await;

                match ticket {
                    Some(ticket) => {
                        ticket.claim(&staff).await;
                        let embed = MessageManager::create_embed(|embed| {
                            embed
                                .description(format!("Claimed by <@{}>", staff))
                            }).await;
                        message.reply(embed).await;
                    },
                    None => message.reply_failure("This channel is not a ticket!").await
                }
            }
        )
    }

}



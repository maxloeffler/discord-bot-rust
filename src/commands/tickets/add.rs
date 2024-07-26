
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct AddMemberToTicketCommand;

impl Command for AddMemberToTicketCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["add".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;
                let member = params.target.unwrap().id;
                let ticket = TicketHandler::get_instance().lock().await
                    .get_ticket(&message.get_channel()).await;

                match ticket {
                    Some(ticket) => {

                        if ticket.present_members.lock().await.contains(&member) {
                            message.reply_failure(&format!("<@{}> is already in this ticket!", member)).await;
                            return;
                        }

                        ticket.add_member(&member).await;
                        let embed = MessageManager::create_embed(|embed| {
                            embed
                                .description(format!("Added <@{}>", member))
                            }).await;
                        let _ = message.reply(embed).await;
                    },
                    None => message.reply_failure("This channel is not a ticket!").await
                }
            }
        )
    }

}




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

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "claim".to_string(),
        ])
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

                        if ticket.present_staff.lock().await.contains(&staff) {
                            message.reply_failure("You have already claimed this ticket!").await;
                            return;
                        }

                        ticket.add_staff(&staff).await;
                        let embed = MessageManager::create_embed(|embed| {
                            embed
                                .description(format!("Claimed by <@{}>", staff))
                            }).await;
                        let _ = message.reply(embed).await;
                    },
                    None => message.reply_failure("This channel is not a ticket!").await
                }
            }
        )
    }

}



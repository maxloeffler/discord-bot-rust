
use serenity::builder::CreateMessage;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct OpenTicketCommand;

impl Command for OpenTicketCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["open".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;
                let target  = &params.target.unwrap();

                // reject if incorrect type
                let ticket_type = message.payload_without_mentions(None, None).await.to_lowercase();
                if ticket_type != "m" && ticket_type != "d" {
                    message.reply_failure("The type of a Ticket can be either `m` (Muted) or `d` (Discussion).").await;
                    return;
                }

                // create ticket
                let ticket = TicketHandler::get_instance().lock().await
                    .new_ticket(message.get_resolver(), target, ticket_type.clone().into()).await;
                if ticket.is_err() {
                    message.reply_failure("Failed to create ticket.").await;
                    return;
                }

                // create embed
                let description = format!("{}\n{}", match ticket_type.as_str() {
                    "m" => "A staff member created this **muted ticket** with you to discuss your warnings.",
                    "d" => "A staff member created this **discussion ticket** with you to discuss a situation you were involved in.",
                    _ => unreachable!(),
                }, "If you **do not** respond within **2 hours**, this ticket will be closed and **appropriate action** will be taken.");
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title("Ticket Created")
                        .description(description)
                }).await;

                // send message
                let embed = CreateMessage::new()
                    .content(format!("<@{}>", target.id))
                    .embed(embed);
                let _ = ticket.unwrap().channel.send_message(message.get_resolver().http(), embed).await;
            }
        )
    }

}



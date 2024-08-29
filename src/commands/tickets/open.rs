
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct OpenTicketCommand;

impl Command for OpenTicketCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Tickets,
            nonempty!["open".to_string()]
        )
            .add_required("user")
            .add_constant("m", false)
            .new_usage()
            .add_required("user")
            .add_constant("d", false)
            .example("@ModAnnoyer m")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = params.message;
                let target  = &params.target.unwrap();

                // reject if incorrect type
                let ticket_type = message.payload_without_mentions(None, None).to_lowercase();
                if ticket_type != "m" && ticket_type != "d" {
                    message.reply_failure("The type of a Ticket can be either `m` (Muted) or `d` (Discussion).").await;
                    return;
                }

                // create ticket
                let ticket = TicketHandler::get_instance()
                    .new_ticket(message.get_resolver(), target, ticket_type.clone().into()).await;
                if ticket.is_err() {
                    message.reply_failure("Failed to create ticket.").await;
                    return;
                }
            }
        )
    }

}



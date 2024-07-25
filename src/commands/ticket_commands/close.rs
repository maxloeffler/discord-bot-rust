
use serenity::builder::CreateMessage;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct CloseTicketCommand;

impl Command for CloseTicketCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["close".to_string()]
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

                        // send close message
                        let embed = MessageManager::create_embed(|embed| {
                            embed
                                .description(format!("Closed by <@{}>", staff))
                            }).await;
                        message.reply(embed).await;

                        // remove ticket from handler
                        TicketHandler::get_instance().lock().await
                            .close_ticket(&ticket.channel.id).await;

                        // obtain channel to dump log
                        let dump_channel = match ticket.ticket_type {
                            TicketType::StaffReport => ConfigDB::get_instance().lock()
                                .await.get("channel_admin").await.unwrap().to_string(),
                            _ => ConfigDB::get_instance().lock()
                                .await.get("channel_transcripts").await.unwrap().to_string()
                        };
                        let dump_channel = message.get_resolver().resolve_channel(&dump_channel).await.unwrap();

                        // produce transcript
                        ticket.transcribe().await;

                        // obtain ticket information
                        let transcript_url = format!(
                            "[External Link](http://thevent.xyz:5000/transcripts/transcript-{}---{}---.html?auth)",
                            ticket.channel.name,
                            ticket.uuid);
                        let staff = ticket.present_staff.lock().await.iter()
                            .map(|id| format!("<@{}>", id))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let members = ticket.present_members.lock().await.iter()
                            .map(|id| format!("<@{}>", id))
                            .collect::<Vec<_>>()
                            .join(", ");

                        // construct log
                        let ticket_type: String = ticket.ticket_type.into();
                        let embed = message.get_log_builder()
                            .title("Ticket Log")
                            .no_thumbnail()
                            .arbitrary("Category", &ticket_type)
                            .arbitrary("Staff", &staff)
                            .arbitrary("Members", &members)
                            .arbitrary("Transcript", &transcript_url)
                            .build().await;
                        let log = CreateMessage::default().embed(embed);

                        // send log
                        let _ = dump_channel.send_message(message.get_resolver().http(), log).await;

                    },
                    None => message.reply_failure("This channel is not a ticket!").await
                }
            }
        )
    }

}



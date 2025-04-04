
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct CloseTicketCommand;

impl Command for CloseTicketCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Tickets,
            nonempty!["close".to_string()]
        )
            .new_usage()
            .add_constant("-tw", false)
            .add_optional("trigger warning")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let staff = &message.get_author().id;
                let ticket = &TicketHandler::get_instance()
                    .get_ticket(&message.get_channel()).await;

                match ticket {
                    Some(ticket) => {

                        // send close message
                        let embed = MessageManager::create_embed(|embed| {
                            embed
                                .description(format!("Closed by <@{}>", staff))
                            }).await;
                        let _ = message.reply(embed).await;

                        // obtain channel to dump log
                        let dump_channel: ChannelId = match ticket.ticket_type {
                            TicketType::StaffReport => ConfigDB::get_instance()
                                .get("channel_admin").await.unwrap().into(),
                            _ => ConfigDB::get_instance()
                                .get("channel_transcripts").await.unwrap().into()
                        };

                        // produce transcript
                        ticket.transcribe().await;

                        // remove ticket from handler
                        TicketHandler::get_instance()
                            .close_ticket(&ticket.channel.id).await;

                        // get the host url (where this bot is hosted)
                        let web_url = ConfigDB::get_instance().get("web_url").await.unwrap();

                        // obtain ticket information
                        let transcript_url = format!(
                            "[External Link]({}/transcripts/transcript-{}---{}---.html?auth)",
                            web_url,
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
                        let mut builder = message.get_log_builder()
                            .title("Ticket Log")
                            .no_thumbnail()
                            .arbitrary("Category", &ticket_type)
                            .arbitrary("Staff", &staff)
                            .arbitrary("Members", &members)
                            .arbitrary("Transcript", &transcript_url);

                        // add field for trigger warning if specified
                        if message.has_parameter("tw") {
                            builder = builder.arbitrary_block(
                                "⚠️ Trigger Warning",
                                message.get_parameter("tw")
                            );
                        }

                        // send log
                        let embed = builder.build().await;
                        let _ = dump_channel.send_message(message, embed.to_message()).await;

                        if ticket.ticket_type == TicketType::Muted {
                            let dms = message.get_author().create_dm_channel(message).await.unwrap();
                            let note = format!("Do not forget to `unmute`, `flag`, or `ban` the member(s) in {}", ticket.channel.name);
                            let _ = dms.send_message(message, note.to_message()).await;
                        }
                    },
                    None => message.reply_failure("This channel is not a ticket!").await
                }
            }
        )
    }

}



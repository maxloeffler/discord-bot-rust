
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::collections::HashSet;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct ReviewCommand;

impl ReviewCommand {

    fn review_ticket<'a>(
        reviewee: User,
        reviewer: &'a User,
        transcript_url: String,
        notes: String,
        dump_channel: ChannelId,
        message: &'a MessageManager) -> BoxedFuture<'a, ()>
    {
        Box::pin(async move {

            if reviewee.id == reviewer.id {
                message.reply_failure("You cannot review yourself.").await;
                return;
            }

            // log review into database
            let approved = message.has_parameter("approve");
            let log = TicketReviewLog::new(
                reviewer.id.to_string(),
                approved,
                notes.clone(),
            );
            TicketReviewsDB::get_instance().lock().await
                .append(&reviewee.id.to_string(), &log.into()).await;

            // create review embed
            let review_amount = TicketReviewsDB::get_instance().lock().await
                .get_all(&reviewee.id.to_string()).await.unwrap().len();
            let transcript_button = CreateButton::new_link(transcript_url).label("Transcript");
            let embed = MessageManager::create_embed(|embed|
                embed
                    .field("Review:", match approved { true => "Approved", false => "Denied" }, false)
                    .field("Notes:",
                        format!("{}\n\n***Please DM <@{}> if you need more information or have any questions.***",
                            notes,
                            reviewer.id.to_string()),
                            false)
                    .footer(CreateEmbedFooter::new(
                            format!("Reviewed by {}", message.get_resolver().resolve_name(reviewer)))
                            .icon_url(reviewer.face()))
            ).await
                .to_message()
                .button(transcript_button)
                .content(format!("<@{}> now has **{}** ticket review(s)", reviewee.id, review_amount));

            let _ = dump_channel.send_message(message, embed).await;
        })
    }

}

impl Command for ReviewCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "review".to_string(),
        ])
            .add_constant("-approve", false)
            .add_optional("notes")
            .new_usage()
            .add_constant("-deny", false)
            .add_optional("notes")
            .example("-approve You did a good job!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let channel = message.get_channel();


                // ---- Sanity Checks ---- //

                let review_channels = ConfigDB::get_instance().lock().await
                    .get_multiple(vec!["channel_suggestions", "channel_transcripts", "channel_admin"]).await.unwrap()
                    .into_iter()
                    .map(|entry| entry.value.to_string())
                    .collect::<Vec<_>>();

                // fail if not in a review channel
                if !review_channels.contains(&channel.to_string()) {
                    let allowed_channels = review_channels.iter()
                        .map(|c| format!("<#{}>", c))
                        .collect::<Vec<_>>()
                        .join(", ");
                    message.reply_failure(&format!(
                            "This command can only be used in {}.",
                            allowed_channels)).await;
                    return;
                }

                if !message.is_referencing() {
                    message.reply_failure("Please reply to the message you want to review.").await;
                    return;
                }
                let referenced_message = message.get_referenced();

                if referenced_message.embeds.is_empty() {
                    message.reply_failure("Pleaes reply to a reviewable message.").await;
                    return;
                }
                let reviewee_embed = &referenced_message.embeds[0];

                if !(message.has_parameter("approve") || message.has_parameter("deny")) {
                    self.invalid_usage(params).await;
                    return;
                }
                let approve = message.has_parameter("approve");

                let mut notes = message.payload(None, Some(vec!["approve".to_string(), "deny".to_string()]));
                if notes.is_empty() {
                    notes = "No notes provided.".to_string();
                }


                // ---- Implementation ---- //

                // In suggestion channel
                if channel.to_string() == review_channels[0] {

                    // extract suggestion and author
                    let suggestion = reviewee_embed.description.clone().unwrap();
                    let original_author = reviewee_embed.footer.clone().unwrap()
                        .text.clone().split("by ").last().unwrap().to_string();

                    let embed = message.get_log_builder()
                        .no_thumbnail()
                        .title(format!("Suggestion {}", match approve { true => "Approved", false => "Denied" }))
                        .arbitrary_block("Suggestion", &suggestion)
                        .arbitrary_block("Review", &notes)
                        .build().await
                        .footer(CreateEmbedFooter::new(
                            format!("Suggested by {} - reviewed by {}", original_author, message.resolve_name()))
                            .icon_url(message.get_author().face()));

                    let edit = EditMessage::new().embed(embed);
                    let _ = channel.edit_message(&message, referenced_message.id, edit).await;

                }

                // in ticket transcript channels
                else {
                    #[cfg(feature = "tickets")]
                    {
                        let resolver = message.get_resolver();
                        let fields = reviewee_embed.fields.iter().collect::<Vec<_>>();

                        // extract staff
                        let regex = RegexManager::get_id_regex();
                        let staff_ids = fields.iter()
                            .find(|field| field.name == "Staff")
                            .unwrap().value.clone()
                            .split(" ")
                            .map(|id| UserId::from(regex.find(&id).unwrap().as_str().parse::<u64>().unwrap()))
                            .collect::<Vec<_>>();
                        let mut staff = HashSet::new();
                        for id in staff_ids {
                            let user = resolver.resolve_user(id).await;
                            if let Some(user) = user {
                                staff.insert(user);
                            }
                        }
                        let staff: Vec<&User> = staff.iter().collect();

                        // extract transcript url
                        let transcript_url = fields.iter()
                            .find(|field| field.name == "Transcript")
                            .unwrap().value.clone()
                            .split("[External Link](").last().unwrap()
                            .split(")").next().unwrap().to_string();

                        let category = TicketType::from(fields.iter()
                            .find(|field| field.name == "Category")
                            .unwrap().value.clone());
                        let dump_channel: ChannelId = match category {
                            TicketType::StaffReport => ConfigDB::get_instance().lock()
                                .await.get("channel_headmod").await.unwrap().into(),
                            _ => ConfigDB::get_instance().lock()
                                .await.get("channel_reviews").await.unwrap().into()
                        };

                        // find reviewer and reviewee, and then call ReviewCommand::review_ticket
                        let reviewer = message.get_author();
                        match staff.len() {
                            0 => unreachable!(),
                            1 => ReviewCommand::review_ticket(staff[0].clone(), reviewer, transcript_url, notes, dump_channel, message).await,
                            _ => {
                                let embed = MessageManager::create_embed(|embed|
                                    embed
                                        .title("Who are you reviewing?")
                                        .description("Select the staff member you are reviewing.")
                                ).await;

                                message.get_interaction_helper()
                                    .create_user_dropdown_interaction(
                                        embed,
                                        staff,
                                        |reviewee: User| ReviewCommand::review_ticket(reviewee, reviewer, transcript_url, notes, dump_channel, message)).await;
                            }
                        };
                    }
                }
            }
        )
    }

}



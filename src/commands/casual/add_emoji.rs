
use serenity::builder::CreateAttachment;
use serenity::utils::parse_emoji;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;


pub struct AddEmojiCommand;

impl Command for AddEmojiCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "add-emoji".to_string(),
            "addemoji".to_string(),
        ])
            .add_required("emoji-id")
            .new_usage()
            .add_required("name")
            .add_required("img-url")
            .example("add-emoji <:pandauwu:1259245515309060238>")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let payload = message.payload(None, None);

                if payload.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                let guild = message.get_resolver().resolve_guild(None).await;
                if let Some(guild) = guild {

                    // obtain emojis information
                    let emojis = guild.emojis.values().collect::<Vec<_>>();
                    let emojis_animated = emojis.iter()
                        .filter(|emoji| emoji.animated).count();
                    let emojis_regular = emojis.iter()
                        .filter(|emoji| !emoji.animated).count();

                    match parse_emoji(&payload) {
                        Some(emoji) => {

                            if emoji.animated && emojis_animated >= 250 {
                                message.reply_failure("Cannot add more animated emojis.").await;
                                return;
                            }

                            if !emoji.animated && emojis_regular >= 250 {
                                message.reply_failure("Cannot add more regular emojis.").await;
                                return;
                            }

                            // parse the emoji to base64
                            let filetype = match emoji.animated {
                                true => "gif",
                                false => "png",
                            };
                            let emoji_url = format!("https://cdn.discordapp.com/emojis/{}.{}", emoji.id, filetype);

                            // create the attachment and upload the emoji
                            let attachment = CreateAttachment::url(&message, &emoji_url).await;
                            if let Ok(attachment) = attachment {
                                let add = guild.create_emoji(&message, &emoji.name, &attachment.to_base64()).await;
                                match add {
                                    Ok(_)  => message.reply_success().await,
                                    Err(_) => message.reply_failure("Failed to add emoji.").await,
                                };
                            }
                        },
                        None => {

                            if emojis_regular >= 250 {
                                message.reply_failure("Cannot add more regular emojis.").await;
                                return;
                            }

                            // extract the url from the payload
                            let mut url  = None;
                            let url_regex = RegexManager::get_url_regex();
                            payload.split_whitespace().into_iter()
                                .for_each(|word| {
                                    if url_regex.is_match(word) {
                                        url = Some(word);
                                    }
                                });

                            // if the url is not found
                            if url.is_none() {
                                self.invalid_usage(params).await;
                                return;
                            }
                            let url = url.unwrap();

                            // everything that is not url is the name
                            let name = message.payload(None, Some(vec![url.to_string()]));
                            if name.is_empty() {
                                self.invalid_usage(params).await;
                                return;
                            }

                            // create the attachment and upload the emoji
                            let attachment = CreateAttachment::url(&message, &url).await;
                            if let Ok(attachment) = attachment {
                                let add = guild.create_emoji(&message, &name, &attachment.to_base64()).await;
                                match add {
                                    Ok(_)  => message.reply_success().await,
                                    Err(_) => message.reply_failure("Failed to add emoji.").await,
                                };
                            }
                        }
                    };
                }
            }
        )
    }
}



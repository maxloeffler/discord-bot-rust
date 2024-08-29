
use serenity::all::ChannelId;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;
use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct PurgeCommand;

impl Command for PurgeCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_mod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "purge".to_string(),
        ])
            .add_required("amount (1..100)")
            .add_optional("user")
            .example("15 @EvilCorp")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let purge_size = params.number.unwrap() as u8;

                if purge_size < 1 || purge_size > 100 {
                    self.invalid_usage(params).await;
                    return;
                }

                if let Some(channel) = message.resolve_guild_channel().await {

                    // check if the category is protected
                    let category_protected_purge = ConfigDB::get_instance()
                        .get_all("category_protected_purge").await.unwrap()
                        .into_iter()
                        .map(|category| category.to_string())
                        .collect::<Vec<_>>();
                    let category = &channel.parent_id.unwrap().to_string();
                    if category_protected_purge.contains(category) {
                        message.reply_failure("You can not purge here.").await;
                        return;
                    }

                    // get messages to delete
                    let mut last_messages = message.last_messages(purge_size).await;
                    let mentions = message.get_mentions().await;
                    if !mentions.is_empty() {
                        last_messages.retain(|message| message.author.id == mentions[0]);
                    }

                    // delete messages
                    let _ = channel.delete_messages(message, last_messages).await;

                    // log to mod logs
                    let embed = message.get_log_builder()
                        .title("[PURGE]")
                        .target(message.get_author())
                        .staff()
                        .arbitrary("Amount", format!("**{}** Message(s)", purge_size))
                        .channel()
                        .timestamp()
                        .build().await;
                    let modlogs: ChannelId = ConfigDB::get_instance()
                        .get("channel_modlogs").await.unwrap().into();
                    let _ = modlogs.send_message(message, embed.to_message()).await;

                    message.reply_success().await;
                }
            }
        )
    }

}

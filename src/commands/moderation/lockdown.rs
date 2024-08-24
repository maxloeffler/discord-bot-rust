
use serenity::model::permissions::Permissions;
use serenity::all::ChannelId;
use serenity::builder::EditChannel;
use nonempty::{NonEmpty, nonempty};
use futures::stream::StreamExt;

use std::cmp::min;
use std::sync::Arc;
use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct LockdownCommand;

impl Command for LockdownCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_mod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "lockdown".to_string(),
        ])
            .new_usage()
            .add_required("-end")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let end_lockdown = message.has_parameter("end");

                let categories: Vec<ChannelId> = ConfigDB::get_instance().lock().await
                    .get_all("category_lockdown").await.unwrap()
                    .into_iter()
                    .map(|category| category.into())
                    .collect();

                // get role id of @everyone
                if let Some(guild) = message.get_guild() {
                    let everyone_role = guild.everyone_role();
                    let notification = message.get_log_builder()
                        .title(if end_lockdown { "Lockdown ended" } else { "Server has been locked down!" })
                        .timestamp()
                        .build().await;

                    // iterate all channels in specified categories
                    for category in categories {
                        let channels = message.get_resolver().resolve_category_channels(category).await.unwrap();
                        futures::stream::iter(channels)
                            .for_each_concurrent(None, |channel| {
                                let notification = notification.clone();
                                async move {

                                    // remove or grant write permissions from everyone
                                    let handler = PermissionHandler::new(&message.get_resolver(), &channel);
                                    match end_lockdown {
                                        true  => handler.allow_role(Permissions::SEND_MESSAGES, &everyone_role).await,
                                        false => handler.deny_role(Permissions::SEND_MESSAGES, &everyone_role).await,
                                    }

                                    // send notification to channel
                                    let _ = channel.send_message(message, notification.to_message()).await;
                                }
                            }).await;
                    }

                    // log lockdown to modlogs
                    let embed = message.get_log_builder()
                        .title(if end_lockdown { "[LOCKDOWN END]" } else { "[LOCKDOWN]" })
                        .staff()
                        .timestamp()
                        .build().await;
                    let modlogs: ChannelId = ConfigDB::get_instance().lock().await
                        .get("channel_modlogs").await.unwrap().into();
                    let _ = modlogs.send_message(message, embed.to_message()).await;

                    message.reply_success().await;
                }
            }
        )
    }

}

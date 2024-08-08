
use serenity::all::ChannelId;
use serenity::builder::EditChannel;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;
use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct SlowmodeCommand;

impl Command for SlowmodeCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "slowmode".to_string(),
            "slow".to_string(),
        ])
            .add_required("delay (0 .. 21600s)")
            .new_usage()
            .add_required("-off")
            .example("slowmode 11")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let payload = message.payload_without_mentions(None, None);

                let mut time_delay = 0;
                match payload.parse::<u16>() {
                    Ok(delay) => time_delay = delay,
                    Err(_) => {
                        if !message.has_parameter("off") {
                            self.invalid_usage(params).await;
                            return;
                        }
                    }
                };

                if time_delay > 21600 {
                    self.invalid_usage(params).await;
                    return;
                }

                if let Some(channel) = message.resolve_guild_channel().await {

                    // check if the category is protected
                    let category_protected_slowmode = ConfigDB::get_instance().lock().await
                        .get_all("category_protected_slowmode").await.unwrap()
                        .into_iter()
                        .map(|category| category.to_string())
                        .collect::<Vec<_>>();
                    let category = &channel.parent_id.unwrap().to_string();
                    if category_protected_slowmode.contains(category) {
                        message.reply_failure("You can not use slowmode here.").await;
                        return;
                    }

                    // set slowmode delay
                    let edit = EditChannel::new()
                        .rate_limit_per_user(time_delay);
                    let _ = message.get_channel().edit(message, edit).await;

                    // log to mod logs
                    let channel_modlogs_id = ConfigDB::get_instance().lock().await
                        .get("channel_modlogs").await.unwrap().to_string();
                    let channel_modlogs = ChannelId::from_str(channel_modlogs_id.as_str()).unwrap();
                    let delay_string = match time_delay {
                        0 => "off".to_string(),
                        _ => format!("{}s", time_delay)
                    };
                    let embed = message.get_log_builder()
                        .title("[SLOWMODE]")
                        .target(message.get_author())
                        .staff()
                        .arbitrary("Delay", delay_string)
                        .channel()
                        .timestamp()
                        .build().await;

                    message.reply_success().await;
                    let _ = channel_modlogs.send_message(message, embed.to_message()).await;
                }
            }
        )
    }

}

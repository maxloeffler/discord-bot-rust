
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct FlagCommand;

impl Command for FlagCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "manual-flag".to_string(),
            "manually-flag".to_string(),
            "manflag".to_string(),
        ])
            .add_required("user")
            .add_optional("reason")
            .add_optional("-monthly")
            .example("manually-flag @GoodGirl repeatedly being bad -monthly")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                // obtain parameters
                let mut reason = message.payload_without_mentions(None, Some(vec!["-monthly".to_string()])).await;
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }
                let monthly = message.has_parameter("monthly");

                // log to database
                let log = FlagLog {
                    member_id: target.id.to_string(),
                    staff_id: message.get_author().id.to_string(),
                    reason: reason,
                    monthly: monthly,
                };
                FlagsDB::get_instance().lock().await
                    .append(&target.id.to_string(), &log.into()).await;

                // log to mod logs
                let channel_modlogs_id = ConfigDB::get_instance().lock().await
                    .get("channel_modlogs").await.unwrap().to_string();
                let channel_modlogs = ChannelId::from_str(&channel_modlogs_id).unwrap();
                let timestamp_now = chrono::Utc::now().timestamp();
                let embed = message.get_log_builder()
                    .title("[FLAG]")
                    .description(&format!("<@{}> has been flagged", target.id))
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .labeled_timestamp("Flag Until", match monthly {
                        true  => timestamp_now + (30 * 24 * 60 * 60),
                        false => timestamp_now + (7  * 24 * 60 * 60)
                    })
                    .build().await;

                message.reply_success().await;
                let _ = channel_modlogs.send_message(message, embed.to_message()).await;
            }
        )
    }

}



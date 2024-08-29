
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::*;
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
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["manually-flag".to_string(), "manual-flag".to_string(), "manflag".to_string()]
        )
            .add_required("user")
            .add_optional("reason")
            .new_usage()
            .add_required("user")
            .add_optional("reason")
            .add_constant("-monthly", false)
            .example("@GoodGirl repeatedly being bad -monthly")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                // obtain parameters
                let mut reason = message.payload_without_mentions(None, Some(vec!["-monthly".to_string()]));
                if reason.is_empty() {
                    reason = "No reason provided.".to_string();
                }
                let monthly = message.has_parameter("monthly");

                // log to database
                let log = FlagLog::new(
                    message.get_author().id.to_string(),
                    reason,
                    monthly,
                );
                FlagsDB::get_instance()
                    .append(&target.id.to_string(), &log.into()).await;

                // log to mod logs
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
                let modlogs: ChannelId = ConfigDB::get_instance()
                    .get("channel_modlogs").await.unwrap().into();
                let _ = modlogs.send_message(message, embed.to_message()).await;

                message.reply_success().await;
            }
        )
    }

}



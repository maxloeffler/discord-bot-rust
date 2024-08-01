
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct UnflagCommand;

impl Command for UnflagCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "unflag".to_string(),
        ])
            .add_required("user")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                let last_flag = FlagsDB::get_instance().lock().await
                    .get_last(&target.id.to_string(), 1).await;

                if let Ok(last_flag) = last_flag {

                    if last_flag.is_empty() {
                        message.reply_failure(&format!("<@{}> has not been flagged.", target.id)).await;
                        return;
                    }

                    // delete last flag
                    FlagsDB::get_instance().lock().await
                        .delete_by_id(last_flag[0].id).await;

                    // log to mod logs
                    let channel_modlogs_id = ConfigDB::get_instance().lock().await
                        .get("channel_modlogs").await.unwrap().to_string();
                    let channel_modlogs = ChannelId::from_str(&channel_modlogs_id).unwrap();
                    let embed = message.get_log_builder()
                        .title("[UNFLAG]")
                        .description(&format!("<@{}> has been unflagged", target.id))
                        .color(0xff8200)
                        .staff()
                        .user(&target)
                        .build().await;

                    message.reply_success().await;
                    let _ = channel_modlogs.send_message(message, embed.to_message()).await;
                }
            }
        )
    }

}



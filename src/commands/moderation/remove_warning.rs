
use serenity::all::*;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemoveWarningCommand;

impl Command for RemoveWarningCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "remove-warn".to_string(),
            "remove-warning".to_string()
        ])
            .add_required("database ID")
            .example("remove-warn 12")
    }

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let warn_id = params.number.unwrap();

                let warn = WarningsDB::get_instance().lock().await
                    .query("", &format!("OR id = {}", warn_id)).await;
                if warn.is_err() || warn.clone().unwrap().is_empty() {
                    message.reply_failure("Warning not found.").await;
                    return;
                }
                let warn = &warn.unwrap()[0];

                // remove warning
                WarningsDB::get_instance().lock().await.delete_by_id(warn_id).await;

                // resolve target
                let log = ModLog::from(warn);
                let user_id = UserId::from(log.member_id.parse::<u64>().unwrap());
                let target = message.get_resolver().resolve_user(user_id).await.unwrap();

                // log to mod logs
                let log_message = message.get_log_builder()
                    .title("[REMOVE WARNING]")
                    .description(&format!("Removed warning with **ID {}**", warn_id))
                    .color(0xff8200)
                    .staff()
                    .user(&target)
                    .timestamp()
                    .build().await;
                let modlogs: ChannelId = ConfigDB::get_instance().lock().await
                    .get("channel_modlogs").await.unwrap().into();
                let _ = modlogs.send_message(message, log_message.to_message()).await;

                message.reply_success().await;
            }
        )
    }

}


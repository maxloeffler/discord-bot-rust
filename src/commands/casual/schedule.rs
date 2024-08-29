
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct ScheduleCommand;

impl Command for ScheduleCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "schedule".to_string(),
            "later".to_string(),
        ])
            .add_required(vec!["time (0..86400s)", "message"])
            .example("60 One minute later!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let time = params.number.unwrap();

                if time < 0 || time > 86400 {
                    self.invalid_usage(params).await;
                    return;
                }

                let content = message.payload(None, Some(vec![time.to_string()]));
                if content.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                // create log
                let log = ScheduleLog::new(
                    chrono::Utc::now().timestamp() + time,
                    content,
                    message.get_channel().to_string(),
                );

                // append log
                ScheduleDB::get_instance()
                    .append(&message.get_author().id.to_string(), &log.into()).await;
                message.reply_success().await;
            }
        )
    }
}



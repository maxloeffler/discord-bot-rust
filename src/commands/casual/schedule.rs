
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct ScheduleCommand;

impl Command for ScheduleCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["schedule".to_string(),"later".to_string()]
        )
            .add_required(vec!["time", "message"])
            .example("1m30s Ninty seconds later!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let time = params.time.unwrap();

                if time > 86_400 {
                    message.reply_failure("Time can at most be 1 day.").await;
                    return;
                }

                let content = message.payload(None, Some(vec![time.to_string()]));
                if content.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                // create log
                let log = ScheduleLog::new(
                    chrono::Utc::now().timestamp() + time as i64,
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



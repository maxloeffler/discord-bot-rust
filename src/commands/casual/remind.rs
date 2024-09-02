
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct RemindCommand;

impl Command for RemindCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["remind".to_string(), "remind-me".to_string(), "reminder".to_string()]
        )
            .add_required(vec!["time", "message"])
            .new_usage()
            .add_constant("-list", false)
            .example("1m30s Ninty later!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let list = message.has_parameter("list");

                // list all reminders
                if list {

                    let reminders = RemindersDB::get_instance()
                        .get_all(&message.get_author().id.to_string()).await.unwrap();
                    let description = match reminders.len() {
                        0 => "You have no reminders.".to_string(),
                        _ => format!("You have **{}** reminders.", reminders.len())
                    };

                    let name = message.resolve_name();
                    let mut builder = message.get_log_builder()
                        .title(&format!("{}' Reminders", name))
                        .description(&description)
                        .no_thumbnail();

                    for reminder in reminders.iter() {
                        builder = builder.schedule_log(&reminder);
                    }

                    let _ = message.reply(builder.build().await).await;
                }

                // create reminder
                else {

                    let time = TimeDecorator::get_time(message).await;
                    if time.is_none() {
                        self.invalid_usage(params).await;
                        return;
                    }
                    let time = time.unwrap();

                    if time > 604_800 {
                        message.reply_failure("Time can at most be 1 week.").await;
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
                    RemindersDB::get_instance()
                        .append(&message.get_author().id.to_string(), &log.into()).await;
                    message.reply_success().await;
                }
            }
        )
    }
}



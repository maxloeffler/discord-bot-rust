
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemindCommand;

impl Command for RemindCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "remind".to_string(),
            "remind-me".to_string(),
            "reminder".to_string(),
        ])
            .add_required("time (0..604800s)")
            .add_required("message")
            .new_usage()
            .add_required("-list")
            .example("remind 60 One minute later!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let list = message.has_parameter("list");

                // list all reminders
                if list {

                    let reminders = RemindersDB::get_instance().lock().await
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

                    let mut time = None;
                    for word in message.words.iter() {
                        if let Ok(time_value) = word.parse::<i64>() {
                            time = Some(time_value);
                            break;
                        }
                    }

                    if time.is_none() {
                        self.invalid_usage(params).await;
                        return;
                    }
                    let time = time.unwrap();

                    // max: 2 weeks
                    if time < 0 || time > 604_800 {
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
                    RemindersDB::get_instance().lock().await
                        .append(&message.get_author().id.to_string(), &log.into()).await;
                    message.reply_success().await;
                }
            }
        )
    }
}




use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RemoveAfkCommand;

impl Command for RemoveAfkCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "remove-afk".to_string()
        ])
            .add_required("user")
    }

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target_id = &params.target.unwrap().id.to_string();

                // check if user is afk
                let afk = AfkDB::get_instance().lock().await
                    .get(&target_id).await;
                if afk.is_err() {
                    message.reply_failure("This user is not afk.").await;
                    return;
                }

                // remove afk message
                AfkDB::get_instance().lock().await
                    .set(&target_id, "This afk message was **removed** by a moderator.").await;

                message.reply_success().await;
            }
        )
    }

}


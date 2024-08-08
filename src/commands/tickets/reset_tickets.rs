
use nonempty::{NonEmpty, nonempty};

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct ResetTicketsCommand;

impl Command for ResetTicketsCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_admin().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "monthly-reset".to_string(),
        ])
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;

                let keys = TicketReviewsDB::get_instance().lock().await
                    .get_keys().await;

                // delete all reviews fom the database
                for key in keys {
                    TicketReviewsDB::get_instance().lock().await
                        .delete(&key).await;
                }

                // delete reviews from local storage
                if cfg!(target_os = "linux") {

                    let directory = fs::read_dir("/var/www/html/transcripts");
                    if let Ok(directory) = directory {

                        // delete all files in the directory
                        for file in directory {
                            if let Ok(file) = file {

                                let path = file.path();
                                if path.is_file() {
                                    fs::remove_file(&path).unwrap();
                                }
                            }
                        }
                    }
                }

                message.reply_success().await;
            }
        )
    }

}



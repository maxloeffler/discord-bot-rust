
use serenity::all::UserId;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct AboutCommand;

impl Command for AboutCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["about".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;

                // get uptime and bot_id from database
                let query = ConfigDB::get_instance().lock().await
                    .get_multiple(vec!["uptime", "bot_id", "command_prefix", "executed_commands"]).await.unwrap();
                let uptime = query[0].to_string().parse::<i64>().unwrap();
                let bot_id = UserId::from(query[1].to_string().parse::<u64>().unwrap());
                let bot = &message.get_resolver().resolve_user(bot_id).await.unwrap();
                let bot_name = &message.get_resolver().resolve_name(bot);

                // create embed
                let embed = message.get_log_builder()
                    .target(bot)
                    .title(&format!("About {}", bot_name))
                    .description(
                        format!(
                            "{} is a powerful Discord bot which runs moderation, tickets, and other miscellaneous tasks for the Kalopsia Discord server.", 
                            bot_name))
                    .labeled_timestamp("Online Since", uptime)
                    .arbitrary("Prefix", format!("`{}`", query[2].to_string()))
                    .arbitrary("Executed Commands", format!("`{}`", query[3].to_string()))
                    .build().await;

                message.reply(embed).await;
            }
        )
    }

}


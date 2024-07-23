
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;


pub struct InfoCommand;

impl Command for InfoCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["info".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target  = &params.target.unwrap();

                let embed = message.get_log_builder()
                    .target(target)
                    .title(&format!("{}' a Information", message.resolve_name()))
                    .arbitrary("Handle", &format!("<@{}>", target.id.to_string()))
                    .labeled_timestamp("Created At", target.created_at().unix_timestamp());

                let member = message.resolve_member().await;
                if let Some(member) = member {

                    if let Some(timestamp) = member.joined_at {
                        embed.clone().labeled_timestamp("Joined At", timestamp.unix_timestamp());
                    }

                    let roles = member.roles.iter()
                        .map(|role_id| format!("<@&{}>", role_id.get()))
                        .collect::<Vec<_>>()
                        .join(", ");

                    match roles.is_empty() {
                        true  => embed.clone().arbitrary("Roles", "None"),
                        false => embed.clone().arbitrary("Roles", roles)
                    };
                }

                message.reply(embed.build().await).await;
            }
        )
    }

}


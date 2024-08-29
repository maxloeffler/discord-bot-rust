
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;


pub struct InfoCommand;

impl Command for InfoCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["info".to_string()]
        )
            .add_required("user")
            .example("@Poggy")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target  = &params.target.unwrap();

                let mut embed = message.get_log_builder()
                    .target(target)
                    .title(&format!("{}'s Information", message.resolve_name()))
                    .arbitrary("Handle", &format!("<@{}>", target.id.to_string()))
                    .labeled_timestamp("Created At", target.created_at().unix_timestamp());

                let member = message.resolve_member().await;
                if let Some(member) = member {

                    if let Some(timestamp) = member.joined_at {
                        embed = embed.labeled_timestamp("Joined At", timestamp.unix_timestamp());
                    }

                    let roles = member.roles.iter()
                        .map(|role_id| format!("<@&{}>", role_id.get()))
                        .collect::<Vec<_>>()
                        .join(", ");

                    embed = match roles.is_empty() {
                        true  => embed.arbitrary("Roles", "None"),
                        false => embed.arbitrary("Roles", roles)
                    };
                }

                let _ = message.reply(embed.build().await).await;
            }
        )
    }

}


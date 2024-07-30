
use serenity::all::ChannelId;
use serenity::builder::EditChannel;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;
use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct RoleCommand;

impl Command for RoleCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "role".to_string(),
        ])
            .add_required("user")
            .add_required("rolenames")
            .example("role @UnhappyCustomer Europe Blue")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.clone().unwrap();

                // get roles
                let payload = message.payload_without_mentions(None, None).await;
                let rolenames = payload
                    .split_whitespace()
                    .collect::<Vec<_>>();
                if rolenames.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                let roles = message.resolve_role(rolenames).await;
                if roles.is_none() {
                    message.reply_failure("Invalid role(s) provided. Please check your spelling!").await;
                    return;
                }

                let member = message.get_resolver().resolve_member(target).await;
                if let Some(member) = member {

                    // add or remove roles
                    for role in roles.unwrap().into_iter() {
                        let has_role = member.roles.contains(&role.id);

                        // remove role if user already has it
                        if has_role {
                            let _ = member.remove_role(&message, &role).await;
                        }

                        // append role if user does not have it
                        else {
                            let _ = member.add_role(&message, &role).await;
                        }

                        // log role update to modlogs
                        let channel_modlogs_id = ConfigDB::get_instance().lock().await
                            .get("channel_modlogs").await.unwrap().to_string();
                        let channel_modlogs = ChannelId::from_str(channel_modlogs_id.as_str()).unwrap();
                        let embed = message.get_log_builder()
                            .title(match has_role {
                                true => "[ROLE REMOVED]",
                                false => "[ROLE ADDED]",
                            })
                            .target(target)
                            .staff()
                            .arbitrary("Role", format!("<@&{}>", &role.id))
                            .timestamp()
                            .build().await;

                        message.reply_success().await;
                        let _ = channel_modlogs.send_message(message, embed.to_message()).await;
                    }
                }
            }
        )
    }

}

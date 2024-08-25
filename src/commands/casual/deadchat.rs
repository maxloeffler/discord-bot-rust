
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};
use chrono::Utc;

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct DeadchatCommand;

impl Command for DeadchatCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "dcp".to_string(),
            "deadchat".to_string()
        ])
            .add_optional("message")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let author = &message.get_author().id.to_string();
                let content = message.payload(None, None);
                let content = match content.is_empty() {
                    true => "".to_string(),
                    false => format!(" `>` {}", content)
                };

                let last_dcp = DeadchatDB::get_instance().lock().await
                    .get_last(author, 1).await.unwrap();

                // You can only dcp every 10 minutes to rate limit pings
                if let Some(dcp) = last_dcp.first() {
                    let now = chrono::Utc::now().timestamp();
                    let next_dcp = dcp.timestamp + 10 * 60;
                    if now < next_dcp {
                        let _ = message.reply_failure(
                            &format!("You can ping deadchat again <t:{}:R>", next_dcp)).await;
                        return;
                    }
                }

                // resolve role and channel
                let role_dcp = message.get_resolver().resolve_role("Dead Chat").await.unwrap()[0].id;
                let channel_welcome: ChannelId = ConfigDB::get_instance().lock().await
                    .get("channel_welcome").await.unwrap().into();

                // create dcp message
                let allowed_mentions = CreateAllowedMentions::new()
                    .roles(vec![role_dcp]);
                let dcp = CreateMessage::new()
                    .content(format!("<@&{}> {} - ***<@{}>***", role_dcp.to_string(), content, author))
                    .allowed_mentions(allowed_mentions);

                // log last dcp
                DeadchatDB::get_instance().lock().await
                    .set(author, &content).await;

                // send to general channel
                let _ = channel_welcome.send_message(&message, dcp).await;

                let _ = message.reply_success().await;
            }
        )
    }
}




use serenity::all::UserId;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct ServerInfoCommand;

impl Command for ServerInfoCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "server-info".to_string(),
            "serverinfo".to_string(),
        ])
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let guild = message.get_resolver().resolve_guild(None).await;

                if let Some(guild) = guild {

                    // obtain emojis information
                    let emojis = guild.emojis.values().collect::<Vec<_>>();
                    let emojis_animated = emojis.iter()
                        .filter(|emoji| emoji.animated).count();
                    let emojis_regular = emojis.iter()
                        .filter(|emoji| !emoji.animated).count();

                    // obtain the bot's user
                    let bot_id = ConfigDB::get_instance().lock().await
                        .get("bot_id").await.unwrap().to_string().parse::<u64>().unwrap();
                    let bot = message.get_resolver().resolve_user(UserId::from(bot_id)).await.unwrap();

                    // obtain the owner
                    let owner = message.get_resolver().resolve_user(guild.owner_id).await.unwrap();
                    let owner_name = message.get_resolver().resolve_name(&owner);

                    // create the embed
                    let embed = message.get_log_builder()
                        .target(&bot)
                        .title(guild.name)
                        .labeled_timestamp("Server Creation", guild.id.created_at().unix_timestamp())
                        .arbitrary("Owner", owner_name)
                        .arbitrary("Roles", guild.roles.len().to_string())
                        .arbitrary("Members", guild.member_count.to_string())
                        .arbitrary("Channels", guild.channels.len().to_string())
                        .arbitrary("Emojis", format!(
                                "Animated: {} / 250\nRegular: {} / 250\nTotal: {} / 500",
                                emojis_animated, emojis_regular, emojis.len()))
                        .build().await;

                    let _ = message.reply(embed).await;
                }
            }
        )
    }

}


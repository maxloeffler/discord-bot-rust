
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};
use chrono::Utc;

use std::iter::FromIterator;
use std::collections::HashMap;

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct HistoryCommand;

impl Command for HistoryCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Moderation,
            nonempty!["history".to_string()]
        )
            .add_required("user")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target  = &params.target.unwrap();

                let mut history = Vec::<(i64, DB, String)>::new();
                let symbols = vec![
                    (DB::Warnings.to_string(), "⚠️ "),
                    (DB::Mutes.to_string(),    "🔇"),
                    (DB::Bans.to_string(),     "🔨"),
                    (DB::Flags.to_string(),    "🚩"),
                ];
                let symbols: HashMap<String, &str> = HashMap::from_iter(symbols.into_iter());

                // fetch moderation logs
                let warnings = WarningsDB::get_instance().get_all(&target.id.to_string()).await.unwrap();
                let mutes    = MutesDB::get_instance().get_all(&target.id.to_string()).await.unwrap();
                let bans     = BansDB::get_instance().get_all(&target.id.to_string()).await.unwrap();
                let flags    = FlagsDB::get_instance().get_all(&target.id.to_string()).await.unwrap();

                // construct correctly sorted history
                warnings.into_iter().for_each(|warning| history.push((warning.timestamp, DB::Warnings, warning.reason)));
                mutes.into_iter().for_each(|mute| history.push((mute.timestamp, DB::Mutes, mute.reason)));
                bans.into_iter().for_each(|ban| history.push((ban.timestamp, DB::Bans, ban.reason)));
                flags.into_iter().for_each(|flag| history.push((flag.timestamp, DB::Flags, flag.reason)));
                history.sort_by(|a, b| a.0.cmp(&b.0));

                // construct description
                let mut description = String::new();
                history.into_iter().for_each(|entry| {
                    description.push_str(&format!("{} <t:{}> `>` {}\n",
                            symbols.get(&entry.1.to_string()).unwrap(),
                            entry.0,
                            entry.2));
                });

                // create embed
                let embed = message.get_log_builder()
                    .title(format!("{}'s Moderation History", message.get_resolver().resolve_name(target)))
                    .target(target)
                    .no_thumbnail()
                    .description(description)
                    .build().await;

                let _ = message.reply(embed).await;
            }
        )
    }

}



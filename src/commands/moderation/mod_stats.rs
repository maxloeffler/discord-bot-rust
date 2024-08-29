
use serenity::all::ChannelId;
use nonempty::{NonEmpty, nonempty};
use chrono::Utc;

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct ModStatsCommand;

impl ModStatsCommand {

    pub fn distribution(logs: Vec<ModLog>) -> (usize, usize, usize) {

        let mut last_day = 0;
        let mut last_week = 0;
        let mut last_month = 0;

        logs.into_iter().rev().for_each(|log| {
            if log.timestamp >= Utc::now().timestamp() - 24 * 60 * 60 {
                last_day += 1;
                last_week += 1;
                last_month += 1;
                return;
            }
            if log.timestamp >= Utc::now().timestamp() - 7 * 24 * 60 * 60 {
                last_week += 1;
                last_month += 1;
                return;
            }
            if log.timestamp >= Utc::now().timestamp() - 30 * 24 * 60 * 60 {
                last_month += 1;
            }
        });

        (last_day, last_week, last_month)
    }

}

impl Command for ModStatsCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_headmod().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "mod-stats".to_string(),
            "modstats".to_string(),
        ])
            .add_required("user")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                // fetch moderation logs
                let warnings = WarningsDB::get_instance()
                    .get_by_staff(&target.id.to_string()).await;
                let mutes = MutesDB::get_instance()
                    .get_by_staff(&target.id.to_string()).await;
                let bans = BansDB::get_instance()
                    .get_by_staff(&target.id.to_string()).await;

                let (warns_last_day, warns_last_week, warns_last_month) = ModStatsCommand::distribution(warnings);
                let (mutes_last_day, mutes_last_week, mutes_last_month) = ModStatsCommand::distribution(mutes);
                let (bans_last_day, bans_last_week, bans_last_month)    = ModStatsCommand::distribution(bans);

                // create embed
                let embed = message.get_log_builder()
                    .title(format!("{}'s Moderation Statistics", message.get_resolver().resolve_name(target)))
                    .target(target)
                    .arbitrary("Last Day",
                        format!("Warnings: **{}**\nMutes: **{}**\nBans: **{}**",
                            warns_last_day, mutes_last_day, bans_last_day))
                    .arbitrary("Last Week",
                        format!("Warnings: **{}**\nMutes: **{}**\nBans: **{}**",
                            warns_last_week, mutes_last_week, bans_last_week))
                    .arbitrary("Last Month",
                        format!("Warnings: **{}**\nMutes: **{}**\nBans: **{}**",
                            warns_last_month, mutes_last_month, bans_last_month))
                    .build().await;

                let _ = message.reply(embed).await;
            }
        )
    }

}



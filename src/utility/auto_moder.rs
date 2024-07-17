
use serenity::model::user::User;

use crate::utility::resolver::Resolver;
use crate::utility::message_manager::MessageManager;
use crate::utility::traits::{Singleton, ToMessage};
use crate::databases::*;


pub struct AutoModerator {}

impl AutoModerator {

    pub fn new() -> Self {
        AutoModerator {}
    }

    pub async fn check_warnings(&self, resolver: Resolver, target: User) {

        // get timestamp of last mute
        let last_mute = MutesDB::get_instance().lock().await
            .get_last(&target.id.to_string(), 1).await;
        let mut last_mute_timestamp = 0;
        if let Ok(last_mute) = last_mute {
            if last_mute.len() > 0 {
                last_mute_timestamp = last_mute[0].timestamp;
            }
        }

        // get all warnings since last mute
        let recent_warnings = WarningsDB::get_instance().lock().await
            .query(&target.id.to_string(),
                   &format!("AND timestamp > {} LIMIT 3", last_mute_timestamp)).await;

        // check if the user has reached 3 warnings
        if recent_warnings.is_ok() && recent_warnings.clone().unwrap().len() == 3 {

            // convert to logs
            let warn_logs: Vec<ModLog> = recent_warnings.clone().unwrap().iter()
                .map(|log| log.into()).collect();


            // mute user
            let role_muted = resolver.resolve_role("Muted").await.unwrap()[0].clone();
            let member = resolver.resolve_member(target.clone()).await.unwrap();
            member.add_role(&resolver.ctx().http, role_muted.id).await.unwrap();

            // create embed
            let embed = MessageManager::create_embed(|embed| {
                embed
                    .title("Automatic Mute")
                    .description(
                        "You have been **automatically muted** because you reached **3** warnings.
                         A staff member will shortly open a **ticket** with you to discuss your warnings.
                         The staff member to delete this note should be the one to create the ticket.")
                    .color(0xFF0000)
            }).await;

            // find person responsible for the last warning (to ping them)
            let last_warning = warn_logs.last().unwrap();
            let bot_id = ConfigDB::get_instance().lock().await
                .get("bot_id").await.unwrap().to_string();
            let role_automute = resolver.resolve_role("Auto Mute").await.unwrap()[0].clone();
            let responsibility = match &last_warning.staff_id {
                bot_id => format!("<@&{}>", role_automute.id),
                _      => format!("<@{}>", last_warning.staff_id)
            };

            // get muted channel
            let muted_channel = ConfigDB::get_instance().lock().await
                .get("channel_muted").await.unwrap().to_string();
            let channel = resolver.resolve_channel(muted_channel).await.unwrap();

            // send 'automute message'
            let _ = channel.send_message(&resolver.ctx().http, responsibility.to_message()).await;
            let _ = channel.send_message(&resolver.ctx().http, embed.to_message()).await;

            // log mute
            let log = ModLog {
                member_id: target.id.to_string(),
                staff_id: bot_id,
                reason: format!("Automatically muted (1: {}, 2: {}, 3: {})",
                    warn_logs[0].reason, warn_logs[1].reason, warn_logs[2].reason),
            };
            MutesDB::get_instance().lock().await
                .append(&target.id.to_string(), &log.into()).await;
        }
    }
}



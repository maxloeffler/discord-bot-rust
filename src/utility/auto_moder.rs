
use serenity::model::user::User;

use crate::utility::*;
use crate::databases::*;


#[cfg(feature = "auto_moderation")]
pub struct AutoModerator {}

#[cfg(feature = "auto_moderation")]
impl AutoModerator {

    pub fn new() -> Self {
        AutoModerator {}
    }

    pub async fn check_warnings(&self, resolver: &Resolver, target: &User) {

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
        let recent_warnings = &WarningsDB::get_instance().lock().await
            .query(&target.id.to_string(),
                   &format!("AND timestamp > {} LIMIT 3", last_mute_timestamp)).await;

        if let Ok(recent_warnings) = recent_warnings {

            // check if the user has reached 3 warnings
            if recent_warnings.len() == 3 {

                // convert to logs
                let warn_logs: Vec<ModLog> = recent_warnings.iter()
                    .map(|log| log.into()).collect();


                // mute user
                let role_muted = &resolver.resolve_role("Muted").await.unwrap()[0];
                let member = resolver.resolve_member(&target).await.unwrap();
                member.add_role(&resolver.ctx().http, role_muted.id).await.unwrap();

                // log mute
                let bot_id = ConfigDB::get_instance().lock().await
                    .get("bot_id").await.unwrap().to_string();
                let log = ModLog {
                    member_id: target.id.to_string(),
                    staff_id: bot_id.clone(),
                    reason: format!("Automatically muted (1: '{}', 2: '{}', 3: '{}')",
                        warn_logs[0].reason, warn_logs[1].reason, warn_logs[2].reason),
                };
                MutesDB::get_instance().lock().await
                    .append(&target.id.to_string(), &log.into()).await;

                // check for active flags
                self.check_mutes(resolver, target).await;

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
                let role_automute = &resolver.resolve_role("Auto Mute").await.unwrap()[0];
                let responsibility = match last_warning.staff_id == bot_id {
                    true  => format!("<@&{}>", role_automute.id),
                    false => format!("<@{}>", last_warning.staff_id)
                };

                // get muted channel
                let channel_muted = ConfigDB::get_instance().lock().await
                    .get("channel_muted").await.unwrap().to_string();
                let channel = resolver.resolve_channel(channel_muted).await.unwrap();

                // send 'automute message'
                let _ = channel.send_message(&resolver.ctx().http, responsibility.to_message()).await;
                let _ = channel.send_message(&resolver.ctx().http, embed.to_message()).await;
            }
        }
    }

    pub async fn check_mutes(&self, resolver: &Resolver, target: &User) {

        // get all flags
        let all_flags = FlagsDB::get_instance().lock().await
            .get_all(&target.id.to_string()).await;

        if let Ok(all_flags) = all_flags {

            // get all active flags
            let active_flags: Vec<FlagLog> = all_flags.iter()
                .filter(|flag| FlagLog::from(*flag).is_active(flag.timestamp))
                .map(|flag| FlagLog::from(flag))
                .collect();

            // if there are active flags
            if active_flags.len() > 0 {

                let flag_reason = active_flags.iter()
                    .enumerate()
                    .map(|(i, flag)| format!("{}: {}", i, flag.reason))
                    .collect::<Vec<String>>()
                    .join(", ");

                // create embed
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title("Flag Notice")
                        .description(format!("<@{}> is currently flagged `>` {}", target.id, flag_reason))
                        .color(0xFF0000)
                }).await;

                // distribute responsibility
                let role_ids = &resolver.resolve_role(vec!["Administrator", "Head Moderator"]).await.unwrap();
                let responsibility = format!("<@&{}> <@&{}>", role_ids[0].id, role_ids[1].id);

                // get muted channel
                let channel_muted = ConfigDB::get_instance().lock().await
                    .get("channel_muted").await.unwrap().to_string();
                let channel = resolver.resolve_channel(channel_muted).await.unwrap();

                // send 'automute message'
                let _ = channel.send_message(&resolver.ctx().http, responsibility.to_message()).await;
                let _ = channel.send_message(&resolver.ctx().http, embed.to_message()).await;
            }
        }
    }

    pub async fn perform_ban(&self, resolver: &Resolver, target: &User, reason: String) {

        let member = &resolver.resolve_member(target).await;
        if let Some(member) = member {

            // ban user
            let success = member.ban_with_reason(&resolver.http(), 0, &reason).await;
            match success {
                Ok(_) => {

                    // log ban to database
                    let bot_id = ConfigDB::get_instance().lock().await
                        .get("bot_id").await.unwrap().to_string();
                    let log = ModLog {
                        member_id: target.id.to_string(),
                        staff_id: bot_id.clone(),
                        reason: format!("Automatically banned ('{}')", reason),
                    };
                    BansDB::get_instance().lock().await
                        .append(&target.id.to_string(), &log.into()).await;

                    // create embed
                    let embed = MessageManager::create_embed(|embed| {
                        embed
                            .title("Automatic Ban")
                            .description(&format!(
                                "{} has been automatically banned for `>` {}",
                                resolver.resolve_name(target),
                                reason))
                            .color(0xFF0000)
                    }).await;

                    // get modlogs channel
                    let channel_id = ConfigDB::get_instance().lock().await
                        .get("channel_modlogs").await.unwrap().to_string();
                    let channel = resolver.resolve_channel(channel_id).await.unwrap();

                    // send 'automute message'
                    let _ = channel.send_message(&resolver.ctx().http, embed.to_message()).await;
                },
                Err(err) => Logger::err_long("Failed to ban user", &err.to_string())
            };
        }
    }
}




use serenity::all::ChannelId;
use serenity::model::user::User;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use std::sync::Arc;
use std::str::FromStr;

use crate::utility::*;
use crate::databases::*;
use crate::impl_singleton;


#[cfg(feature = "auto_moderation")]
pub struct AutoModerator {}

#[cfg(feature = "auto_moderation")]
impl_singleton!(AutoModerator);

#[cfg(feature = "auto_moderation")]
impl AutoModerator {

    pub fn new() -> Self {
        AutoModerator {}
    }

    // ---- Check methods ---- //

    pub async fn check_warnings(&self, message: &MessageManager, target: &User) {

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
                let reason = recent_warnings.iter()
                    .map(|warning| warning.reason.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.perform_mute(message, target, reason).await;
            }
        }
    }

    pub async fn check_flags(&self, message: &MessageManager, target: &User) {

        let resolver = message.get_resolver();

        // get all flags
        let all_flags = FlagsDB::get_instance().lock().await
            .get_all(&target.id.to_string()).await;

        if let Ok(all_flags) = all_flags {

            // get all active flags
            let active_flags: Vec<FlagLog> = all_flags
                .into_iter()
                .filter(|flag| flag.is_active(flag.timestamp))
                .collect();

            // if there are active flags
            if active_flags.len() > 0 {

                let flag_reason = active_flags.into_iter()
                    .map(|flag| flag.reason)
                    .collect::<Vec<String>>()
                    .join(", ");
                let target_id = target.id.to_string();

                // create embed
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title("Flag Notice")
                        .description(format!("<@{}> is currently flagged `>` {}", target_id, flag_reason))
                        .color(0xFF0000)
                }).await;

                // distribute responsibility
                let role_ids = &resolver.resolve_role(vec!["Administrator", "Head Moderator"]).await.unwrap();
                let responsibility = format!("<@&{}> <@&{}>", role_ids[0].id, role_ids[1].id);

                // get muted channel
                let channel: ChannelId = ConfigDB::get_instance().lock().await
                    .get("channel_muted").await.unwrap().into();

                // send flag notice
                let _ = channel.send_message(resolver, responsibility.to_message()).await;
                let _ = channel.send_message(resolver, embed.to_message()).await;
            }
        }
    }

    // ---- Perform methods ---- //

    pub async fn perform_warn(&self, message: &MessageManager, target: &User, reason: String, context: String) {

        let resolver = message.get_resolver();
        let target_id = target.id.to_string();

        // warn user
        let warn_message = format!("<@{}>, you have been **automatically warned** `>` {}",
            target_id,
            reason);
        let _ = message.reply(warn_message.to_message()).await;

        let bot_id = ConfigDB::get_instance().lock().await
            .get("bot_id").await.unwrap().to_string();

        // log to database
        let log = ModLog::new(
            bot_id,
            context.clone()
        );
        WarningsDB::get_instance().lock().await
            .append(&target_id, &log.into()).await;

        // log to mod logs
        let log_message = message.get_log_builder()
            .title("[AUTOMATIC WARNING]")
            .target(&target)
            .description(&format!("<@{}> has been automatically warned", target_id))
            .color(0xff8200)
            .user(&target)
            .arbitrary("Reason", &context)
            .timestamp()
            .build().await;
        let modlogs: ChannelId = ConfigDB::get_instance().lock().await
            .get("channel_modlogs").await.unwrap().into();
        let _ = modlogs.send_message(resolver, log_message.to_message()).await;

        // check if the user has been warned too many times
        self.check_warnings(message, &target).await;
    }

    pub async fn perform_mute(&self, message: &MessageManager, target: &User, reason: String) {

        let resolver = message.get_resolver();
        let target_id = target.id.to_string();

        // mute user
        let role_muted = &resolver.resolve_role("Muted").await.unwrap()[0];
        let member = resolver.resolve_member(&target).await.unwrap();
        member.add_role(&resolver, role_muted.id).await.unwrap();

        // log mute to database
        let bot_id = ConfigDB::get_instance().lock().await
            .get("bot_id").await.unwrap().to_string();
        let log = ModLog::new(
            bot_id.clone(),
            reason.clone(),
        );
        MutesDB::get_instance().lock().await
            .append(&target.id.to_string(), &log.into()).await;

        // log mute to modlogs
        let log_message = message.get_log_builder()
            .title("[AUTOMATIC MUTE]")
            .target(&target)
            .description(&format!("<@{}> has been automatically muted", target_id))
            .user(&target)
            .arbitrary("Reason", reason)
            .timestamp()
            .build().await;
        let modlogs: ChannelId = ConfigDB::get_instance().lock().await
            .get("channel_modlogs").await.unwrap().into();
        let _ = modlogs.send_message(message.get_resolver(), log_message.to_message()).await;

        // check for active flags
        self.check_flags(message, target).await;

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
        let last_warning = &WarningsDB::get_instance().lock().await
            .get_last(&target_id, 1).await.unwrap()[0];
        let role_automute = &resolver.resolve_role("Auto Mute").await.unwrap()[0];
        let responsibility = match last_warning.staff_id == bot_id {
            true  => format!("<@&{}>", role_automute.id),
            false => format!("<@{}>", last_warning.staff_id)
        };

        // get muted channel
        let channel: ChannelId = ConfigDB::get_instance().lock().await
            .get("channel_muted").await.unwrap().into();

        // send automute message
        let _ = channel.send_message(resolver, responsibility.to_message()).await;
        let _ = channel.send_message(resolver, embed.to_message()).await;
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
                    let log = ModLog::new(
                        bot_id.clone(),
                        format!("Automatically banned ('{}')", reason),
                    );
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
                    let channel: ChannelId = ConfigDB::get_instance().lock().await
                        .get("channel_modlogs").await.unwrap().into();

                    // send autoban message
                    let _ = channel.send_message(resolver, embed.to_message()).await;
                },
                Err(err) => Logger::err_long("Failed to ban user", &err.to_string())
            };
        }
    }
}




use serenity::model::channel::ReactionType::{Unicode, Custom};
use serenity::all::*;
use serenity::model::id::ChannelId;
use serenity::builder::{CreateWebhook, CreateAttachment, ExecuteWebhook, CreateAllowedMentions};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use strum::IntoEnumIterator;
use chrono::Utc;
use futures::stream::StreamExt;

use std::sync::Arc;
use std::thread;
use std::str::FromStr;

use crate::databases::*;
use crate::utility::*;


pub async fn spawn(thread: BoxedFuture<'static, ()>) {
    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(thread);
    });
}

#[cfg(feature = "db_interface")]
pub fn database_interface<'a>() -> BoxedFuture<'a, ()> {
    Box::pin(async move {
        let mut db = DB::Config;
        Logger::info_long("Connected to database", db.to_string().as_str());
        loop {
            let database = ConfigDB::get_instance();
            let input = Logger::input("Enter a command");
            let words = input.split_whitespace().collect::<Vec<&str>>();

            match words[0] {
                "ls" => {
                    let mut keys = database.get_keys().await;
                    keys.sort();
                    Logger::info_long("Keys", &keys.join(", "));
                }
                "get" => {
                    match words.len() {
                        1 => {
                            Logger::warn("Too few parameters");
                        },
                        2 => {
                            let key = words[1];
                            let value = database.get(key).await;
                            match value {
                                Ok(value) => Logger::info_long(&format!("Value of {}", key), &value.to_string()),
                                Err(err) => Logger::err(err.as_str())
                            }
                        }
                        _ => {
                            match words[1] {
                                "all" => {
                                    let values = database.get_all(words[2]).await;
                                    match values {
                                        Ok(values) => {
                                            let values: Vec<_> = values.iter().map(|entry| entry.to_string()).collect();
                                            Logger::info_long(&format!("Values of {}", words[2]), &values.join(", "))
                                        }
                                        Err(err) => Logger::err(err.as_str())
                                    }
                                },
                                _ => {
                                    let values = database.get_multiple(words[1..].to_vec()).await;
                                    match values {
                                        Ok(values) => {
                                            let values: Vec<_> = values.iter().map(|entry| entry.to_string()).collect();
                                            Logger::info_long(&format!("Values of {}", &words[1..].join(", ")), &values.join(", "))
                                        }
                                        Err(err) => Logger::err(err.as_str())
                                    }
                                }
                            }
                        }
                    }
                }
                "set" => {
                    match words.len() {
                        1..=2 => {
                            Logger::warn("Too few parameters");
                        }
                        3 => {
                            let key = words[1];
                            let value = words[2];
                            database.set(key, value).await;
                            Logger::info_long(&format!("Set value for {}", key), value);
                        }
                        _ => {
                            let _key = words[1];
                            let _values = &words[2..];
                            Logger::warn("Currently not implemented!");
                        }
                    }
                }
                "rm" => {
                    match words.len() {
                        2 => {
                            let key = words[1];
                            database.delete(key).await;
                            Logger::info_long("Removed key", key);
                        }
                        _ => {
                            Logger::warn("Too many parameters");
                        }
                    }
                },
                "append" => {
                    match words.len() {
                        1..=2 => {
                            Logger::warn("Too few parameters")
                        }
                        3 => {
                            let key = words[1];
                            let value = words[2];
                            database.append(key, value).await;
                            Logger::info_long(&format!("Appended value to {}", key), value);
                        }
                        _ => {
                            let _key = words[1];
                            let _values = &words[2..];
                            Logger::warn("Currently not implemented!");
                        }
                    }
                }
                "cd" => {
                    match words.len() {
                        2 => {
                            let mut switch = false;
                            for db_type in DB::iter() {
                                if db_type.to_string() == words[1] {
                                    switch = true;
                                    db = db_type;
                                }
                            }
                            match switch {
                                true => Logger::info_long("Switched to database", db.to_string().as_str()),
                                _    => Logger::warn("Invalid database")
                            }
                        }
                        _ => {
                            Logger::warn("Too many parameters");
                        }
                    }
                }
                _ => {
                    Logger::err("Invalid command");
                }
            }
        }
    })
}

pub fn periodic_checks<'a>(resolver: Resolver) -> BoxedFuture<'a, ()> {
    Box::pin(async move {
        let resolver = &resolver;
        let allowed_mentions = &CreateAllowedMentions::new();
        loop {

            // check for scheduled messages
            let users = ScheduleDB::get_instance().get_keys().await;
            let now = chrono::Utc::now().timestamp();

            // remove all pending webhooks
            if let Some(guild) = resolver.resolve_guild(None).await {
                let webhooks = guild.webhooks(resolver).await;
                if let Ok(webhooks) = webhooks {
                    for webhook in webhooks {
                        webhook.delete(resolver).await.unwrap();
                    }
                }
            }

            // for all users that have scheduled messages
            futures::stream::iter(users)
                .map(|user| UserId::from(user.parse::<u64>().unwrap()))
                .for_each_concurrent(None, |user| {
                    async move {

                        // get scheduled messages
                        let scheduled_messages = ScheduleDB::get_instance()
                            .get_all(&user.to_string()).await;
                        let user = resolver.resolve_user(user).await.unwrap();

                        // for all scheduled messages
                        if let Ok(scheduled_messages) = scheduled_messages {
                            for scheduled_message in scheduled_messages.into_iter() {

                                // check if message is expired
                                if scheduled_message.is_expired(now) {

                                    // delete scheduled message from database
                                    ScheduleDB::get_instance().delete_by_id(scheduled_message.id).await;

                                    // create webhook
                                    let channel_id = ChannelId::from_str(&scheduled_message.channel_id).unwrap();
                                    let hook = channel_id.create_webhook(resolver,
                                        CreateWebhook::new(resolver.resolve_name(&user))
                                            .avatar(&CreateAttachment::url(resolver, &user.face()).await.unwrap())
                                    ).await.unwrap();

                                    // create embed
                                    let execute = ExecuteWebhook::new()
                                        .content(scheduled_message.message)
                                        .allowed_mentions(allowed_mentions.clone());
                                    let _ = hook.execute(resolver, false, execute).await;
                                }
                            }
                        }
                    }
                }).await;


            // check for reminders
            let users = RemindersDB::get_instance().get_keys().await;
            let now = chrono::Utc::now().timestamp();

            // for all users that have scheduled messages
            futures::stream::iter(users)
                .map(|user| UserId::from(user.parse::<u64>().unwrap()))
                .for_each_concurrent(None, |user| {
                    async move {

                        // get scheduled messages
                        let reminders = RemindersDB::get_instance().get_all(&user.to_string()).await;
                        let user = resolver.resolve_user(user).await.unwrap();

                        // for all reminders
                        if let Ok(reminders) = reminders {
                            for reminder in reminders.into_iter() {

                                // check if message is expired
                                if reminder.is_expired(now) {

                                    // delete scheduled message from database
                                    RemindersDB::get_instance().delete_by_id(reminder.id).await;

                                    // create embed
                                    let embed = MessageManager::create_embed(|embed| {
                                        embed
                                            .title("Reminder")
                                            .description(reminder.message)
                                            .color(0x00FF00)
                                    }).await;
                                    let embed = CreateMessage::new()
                                        .content(format!("<@{}>", user.id.to_string()))
                                        .embed(embed);

                                    let channel = ChannelId::from_str(&reminder.channel_id).unwrap();
                                    let _ = channel.send_message(resolver, embed).await;
                                }
                            }
                        }
                    }
                }).await;

            // clean message logs
            #[cfg(feature = "message_logs")]
            {
                let channel_id: ChannelId = ConfigDB::get_instance().get("channel_messagelogs").await.unwrap().into();
                let channel = resolver.resolve_guild_channel(channel_id).await.unwrap();

                let mut last_message = channel.last_message_id.unwrap();
                let mut count = 500;

                while count > 0 {
                    let messages = channel.messages(resolver, GetMessages::new().before(last_message).limit(100)).await.unwrap();
                    if messages.len() == 0 {
                        break;
                    }
                    last_message = messages.last().unwrap().id;
                    count -= messages.len();
                }

                let builder = GetMessages::new().before(last_message).limit(100);
                let messages = channel.messages(resolver, builder).await;
                match messages {
                    Ok(messages) => { let _ = channel_id.delete_messages(resolver, messages).await; },
                    Err(_) => {},
                };
            }

            // remind staff if last message in ticket is by a member
            // and longer than 10 minutes ago
            #[cfg(feature = "tickets")]
            {

                let tickets = TicketHandler::get_instance().tickets.read().unwrap().clone();
                let _ = futures::stream::iter(tickets.values())
                    .for_each_concurrent(None, |ticket| async {

                        if !(*ticket.pinged_staff.lock().await) {
                            if let Some(message_id) = ticket.channel.last_message_id {
                                let message = resolver.resolve_message(ticket.channel.id, message_id).await.unwrap();

                                // if last message is by a member and is older than 10 minutes
                                if ticket.present_members.lock().await.contains(&message.author.id)
                                    && message.timestamp.timestamp() + 600 < Utc::now().timestamp() {
                                    ticket.ping_staff().await;
                                }

                                // if last message is by staff
                                else if ticket.present_staff.lock().await.contains(&message.author.id) {
                                    ticket.reset_ping().await;
                                }
                            }
                        }
                }).await;
            }
        }
    })
}

#[cfg(feature = "tickets")]
pub fn hook_ticket_selector<'a>(resolver: Resolver, channel: GuildChannel) -> BoxedFuture<'a, ()> {
    Box::pin(async move {

        // listen for reactions
        let resolver = &resolver;
        let mut last_reaction = (None, chrono::Utc::now().timestamp());
        let mut reactions = channel
            .await_reaction(&resolver.ctx().shard)
            .stream();

        while let Some(reaction) = reactions.next().await {

            let _ = reaction.delete(&resolver).await;

            // cannot handle tickets if user is not available
            if reaction.user_id.is_none() {
                continue;
            }
            let target = resolver.resolve_user(reaction.user_id.unwrap()).await.unwrap();

            match reaction.emoji {
                Unicode(ref emoji) => {

                    // same user cannot create multiple tickets within 10 seconds
                    if last_reaction.0.is_some()
                        && last_reaction.0.unwrap() == target.id
                        && last_reaction.1 + 10 > chrono::Utc::now().timestamp() {
                        continue;
                    }

                    // update last_reaction
                    last_reaction = (reaction.user_id, chrono::Utc::now().timestamp());

                    // parse emoji
                    let ticket_type = match emoji.as_str() {
                        "📁" => Some(TicketType::StaffReport),
                        "💼" => Some(TicketType::UserReport),
                        "📔" => Some(TicketType::BugReport),
                        "🤔" => Some(TicketType::Question),
                        _ => None
                    };

                    // invalid reaction
                    if ticket_type.is_none() {
                        continue;
                    }

                    let _ = TicketHandler::get_instance()
                        .new_ticket(resolver, &target, ticket_type.unwrap()).await;
                },
                _ => {}
            }
        }
    })
}


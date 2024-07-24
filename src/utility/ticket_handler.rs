
use serenity::model::channel::GuildChannel;
use serenity::model::id::{ChannelId, RoleId, UserId};
use serenity::builder::GetMessages;
use serenity::model::prelude::*;
use serenity::prelude::*;
use futures::stream::StreamExt;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use crate::utility::*;
use crate::databases::*;


pub struct Ticket {
    pub channel: GuildChannel,
    pinged_staff: bool,

    present_members: Arc<Mutex<HashSet<User>>>,
    present_staff: Arc<Mutex<HashSet<User>>>,
    allowed_roles: Vec<RoleId>,
}

pub struct TicketHandler {
    tickets: Arc<Mutex<HashMap<String, Ticket>>>,
}

impl TicketHandler {

    pub fn new() -> Self {
        TicketHandler {
            tickets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn init(&self, resolver: &Resolver) {

        #[cfg(feature = "debug")]
        Logger::info_long("Start", "Initilizing ticket handler");

        // get all ticket channels
        let channels = resolver.resolve_guild_channels().await;

        if let Some(channels) = channels {

            // get the ticket category
            let ticket_category = ConfigDB::get_instance().lock().await
                .get("category_tickets").await.unwrap().to_string();

            // recover all tickets
            let tickets = Arc::new(Mutex::new(HashMap::new()));
            let parse = futures::stream::iter(channels.iter()
                .filter(|channel| {
                    let category = channel.parent_id;
                    if let Some(category) = category {
                        return category.to_string() == ticket_category
                    }
                    false
                })
                .collect::<Vec<&GuildChannel>>())
                .for_each_concurrent(None, |channel| {
                    let tickets = Arc::clone(&tickets);
                    async move {
                        let ticket = Ticket::parse_ticket(resolver, channel).await;
                        tickets.lock().await.insert(channel.id.to_string(), ticket);
                    }
                });
            parse.await;
        }

        #[cfg(feature = "debug")]
        Logger::info_long("End", "Initilizing ticket handler");

    }
}

impl Default for Ticket {
    fn default() -> Self {
        Ticket {
            channel: GuildChannel::default(),
            pinged_staff: false,
            present_members: Arc::new(Mutex::new(HashSet::new())),
            present_staff: Arc::new(Mutex::new(HashSet::new())),
            allowed_roles: Vec::new(),
        }
    }
}

impl Ticket {

    pub async fn parse_ticket(resolver: &Resolver, channel: &GuildChannel) -> Self {

        #[cfg(feature = "debug")]
        Logger::info_long("Parsing ticket", &channel.name);

        let last_message = channel.last_message_id;
        if let Some(last_message) = last_message {

            // get all messages in the channel
            let builder = GetMessages::new().before(last_message).limit(255);
            let messages = channel.messages(&resolver.http(), builder).await;

            if let Ok(messages) = messages {
                let first_message = messages.first().unwrap();

                // get first message in ticket
                let message = resolver.resolve_message(channel.id, first_message.id).await.unwrap();
                let target_id = message.author.id;
                let target = resolver.resolve_user(target_id).await;

                // get all allowed staff roles for the ticket
                let allowed_roles = message.mention_roles;
                let allowed_staff_roles = match allowed_roles.len() {
                    0 => {
                        resolver.resolve_role(vec!["Trial Moderator", "Moderator", "Head Moderator", "Administrator"]).await
                            .unwrap().iter()
                            .map(|role| role.id)
                            .collect::<Vec<RoleId>>()
                    },
                    _ => allowed_roles,
                };

                // get all present staff and members in the ticket
                let present_staff = Arc::new(Mutex::new(HashSet::new()));
                let present_members = Arc::new(Mutex::new(HashSet::new()));
                let message_iter = futures::stream::iter(messages)
                    .for_each_concurrent(None, |message: Message| {
                        let present_staff = Arc::clone(&present_staff);
                        let present_members = Arc::clone(&present_members);
                        async move {
                            let author = message.author;
                            if !author.bot {
                                match resolver.is_trial(&author).await {
                                    true => present_staff.lock().await.insert(author),
                                    false => present_members.lock().await.insert(author),
                                };
                            }
                        }});
                message_iter.await;

                // target should always be present in the ticket
                if let Some(target) = target {
                    present_members.lock().await.insert(target);
                }

                return Ticket {
                    channel: channel.clone(),
                    pinged_staff: false,
                    present_members: present_members,
                    present_staff: present_staff,
                    allowed_roles: allowed_staff_roles,
                };
            }
        }
        Ticket::default()
    }
}


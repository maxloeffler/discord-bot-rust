
use serenity::model::permissions::Permissions;
use serenity::model::channel::GuildChannel;
use serenity::model::id::{ChannelId, RoleId, UserId};
use serenity::builder::{CreateChannel, GetMessages};
use serenity::model::prelude::*;
use serenity::prelude::*;
use futures::stream::StreamExt;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use crate::utility::*;
use crate::databases::*;


#[derive(Debug, Copy, Clone)]
pub enum TicketType {
    Muted,
    Discussion,
    Question,
    BugReport,
    UserReport,
    StaffReport,
}

impl Into<String> for TicketType {
    fn into(self) -> String {
        match self {
            TicketType::Muted => "Muted".to_string(),
            TicketType::Discussion => "Discussion".to_string(),
            TicketType::Question => "Question".to_string(),
            TicketType::BugReport => "Bug Report".to_string(),
            TicketType::UserReport => "User Report".to_string(),
            TicketType::StaffReport => "Staff Report".to_string(),
        }
    }
}

impl From<String> for TicketType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Muted"      | "m"  => TicketType::Muted,
            "Discussion" | "d"  => TicketType::Discussion,
            "Question"          => TicketType::Question,
            "Bug Report"        => TicketType::BugReport,
            "User Report"       => TicketType::UserReport,
            "Staff Report"      => TicketType::StaffReport,
            _ => TicketType::Discussion,
        }
    }
}

pub struct Ticket {
    pub channel: GuildChannel,
    pub ticket_type: TicketType,
    pub resolver: Resolver,
    pub pinged_staff: bool,

    pub present_members: Arc<Mutex<HashSet<UserId>>>,
    pub present_staff: Arc<Mutex<HashSet<UserId>>>,
    pub allowed_roles: Vec<RoleId>,
}

impl Debug for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ticket_type: String = self.ticket_type.into();
        write!(f, "Ticket ({}): {}", ticket_type, self.channel.name)
    }
}

pub struct TicketHandler {
    tickets: Arc<Mutex<HashMap<String, Arc<Ticket>>>>,
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
                        if let Ok(ticket) = ticket {
                            let ticket = Arc::new(ticket);
                            tickets.lock().await.insert(channel.id.to_string(), ticket);
                        }
                    }
                });
            parse.await;
        }

        #[cfg(feature = "debug")]
        Logger::info_long("End", "Initilizing ticket handler");

    }

    pub async fn new_ticket(&self,
            resolver: &Resolver,
            target: &User,
            ticket_type: TicketType) -> Result<Arc<Ticket>>
    {

        #[cfg(feature = "debug")]
        Logger::info_long("Start", "Creating new ticket");

        // resolve guild
        let guild = resolver.resolve_guild(None).await;
        if let Some(guild) = guild {

            // get the ticket category
            let ticket_category = ConfigDB::get_instance().lock().await
                .get("category_tickets").await.unwrap().to_string();
            let ticket_category = ticket_category.parse::<u64>().unwrap();

            // create new channel
            let builder = CreateChannel::new(resolver.resolve_name(target))
                .category(ChannelId::from(ticket_category))
                .topic(ticket_type);
            let channel = guild.create_channel(resolver.http(), builder).await;

            if let Ok(channel) = channel {

                // figure out allowed and disallowed roles
                let mut allowed_roles = vec!["Administrator", "Head Moderator", "Moderator", "Trial Moderator"];
                let mut disallowed_roles = vec![];
                match ticket_type {
                    TicketType::StaffReport => {
                        let trial = allowed_roles.pop().unwrap();
                        disallowed_roles.push(trial);
                    },
                    TicketType::UserReport => {
                        let trial = allowed_roles.pop().unwrap();
                        let moderator = allowed_roles.pop().unwrap();
                        disallowed_roles.push(moderator);
                        disallowed_roles.push(trial);
                    },
                    _ => {},
                };

                let allowed_roles = resolver.resolve_role(allowed_roles).await.unwrap().iter()
                    .map(|role| role.id)
                    .collect::<Vec<RoleId>>();
                let disallowed_roles = resolver.resolve_role(disallowed_roles).await.unwrap().iter()
                    .map(|role| role.id)
                    .collect::<Vec<RoleId>>();

                // remove explicitly disallowed roles
                let handler = PermissionHandler::new(resolver, &channel);
                handler.deny_role(&Permissions::VIEW_CHANNEL, disallowed_roles).await;

                let present_members = Arc::new(Mutex::new(HashSet::new()));
                present_members.lock().await.insert(target.id);

                // create ticket
                let ticket = Arc::new(Ticket {
                    channel: channel,
                    ticket_type: ticket_type,
                    resolver: resolver.clone(),
                    pinged_staff: false,

                    present_members: present_members,
                    present_staff: Arc::new(Mutex::new(HashSet::new())),
                    allowed_roles: allowed_roles.clone(),
                });

                ticket.allow_participants().await;
                self.tickets.lock().await.insert(ticket.channel.to_string(), Arc::clone(&ticket));

                return Ok(ticket);
            }
        }

        #[cfg(feature = "debug")]
        Logger::info_long("End", "Creating new ticket");

        Err("Failed to create ticket".into())

    }

    pub async fn close_ticket(&self, channel: &ChannelId) {
        let ticket = self.get_ticket(channel).await;
        if let Some(ticket) = ticket {
            ticket.deny_all().await;
            let _ = ticket.channel.delete(&ticket.resolver.http()).await;
        }
    }

    pub async fn get_ticket(&self, channel: &ChannelId) -> Option<Arc<Ticket>> {
        let tickets = self.tickets.lock().await;
        match tickets.get(&channel.to_string()) {
            Some(ticket) => {
                let ticket = Arc::clone(ticket);
                Some(ticket)
            },
            None => None,
        }
    }
}

impl Ticket {

    pub fn get_permissions<'a>(&'a self) -> PermissionHandler<'a> {
        PermissionHandler::new(&self.resolver, &self.channel)
    }

    pub async fn parse_ticket(resolver: &Resolver, channel: &GuildChannel) -> Result<Ticket> {

        #[cfg(feature = "debug")]
        Logger::info_long("Parsing ticket", &channel.name);

        let last_message = channel.last_message_id;
        if let Some(last_message) = last_message {

            // get all messages in the channel
            let builder = GetMessages::new().around(last_message).limit(255);
            let messages = channel.messages(&resolver.http(), builder).await;

            if let Ok(messages) = messages {

                // get all present staff and members in the ticket
                let present_staff = Arc::new(Mutex::new(HashSet::new()));
                let present_members = Arc::new(Mutex::new(HashSet::new()));
                let message_iter = futures::stream::iter(messages.clone())
                    .for_each_concurrent(None, |message: Message| {
                        let present_staff = Arc::clone(&present_staff);
                        let present_members = Arc::clone(&present_members);
                        async move {
                            let author = message.author;
                            if !author.bot {
                                match resolver.is_trial(&author).await {
                                    true => present_staff.lock().await.insert(author.id),
                                    false => present_members.lock().await.insert(author.id),
                                };
                            }
                        }});
                message_iter.await;

                // target should always be present in the ticket
                let first_message = messages.last().unwrap();
                let first_message = &MessageManager::new(resolver.clone(), first_message.clone()).await;
                let mentions = first_message.get_mentions().await;
                present_members.lock().await.insert(mentions[0]);

                // get all allowed staff roles for the ticket
                let mut allowed_roles = resolver.resolve_role(
                    vec!["Administrator", "Head Moderator", "Moderator", "Trial Moderator"])
                        .await.unwrap().iter()
                        .map(|role| role.id)
                        .collect::<Vec<RoleId>>();

                // get mentioned roles
                let role_mentions = first_message.get_mentioned_roles().await;
                if role_mentions.len() > 0 {
                    allowed_roles = role_mentions;
                }

                // get ticket type
                let ticket_type: TicketType = channel.topic.clone().unwrap_or("Discussion".to_string()).into();

                // create ticket
                let ticket = Ticket {
                    channel: channel.clone(),
                    ticket_type: ticket_type,
                    resolver: resolver.clone(),
                    pinged_staff: false,

                    present_members: present_members,
                    present_staff: present_staff,
                    allowed_roles: allowed_roles.clone(),
                };

                // setup permissions
                let handler = ticket.get_permissions();
                handler.deny_role(&Permissions::SEND_MESSAGES, allowed_roles).await;
                ticket.allow_participants().await;

                return Ok(ticket);
            }
        }
        Err("Failed to parse ticket".into())
    }

    pub async fn allow_participants(&self) {
        let handler = self.get_permissions();
        for user_id in self.present_members.lock().await.iter() {
            handler.allow_member(&Permissions::SEND_MESSAGES, user_id).await;
            handler.allow_member(&Permissions::VIEW_CHANNEL, user_id).await;
        }
        for user_id in self.present_staff.lock().await.iter() {
            handler.allow_member(&Permissions::SEND_MESSAGES, user_id).await;
            handler.allow_member(&Permissions::VIEW_CHANNEL, user_id).await;
        }
    }

    pub async fn deny_all(&self) {
        let handler = self.get_permissions();
        for user_id in self.present_members.lock().await.iter() {
            handler.deny_member(&Permissions::SEND_MESSAGES, user_id).await;
            handler.deny_member(&Permissions::VIEW_CHANNEL, user_id).await;
        }
        for user_id in self.present_staff.lock().await.iter() {
            handler.deny_member(&Permissions::SEND_MESSAGES, user_id).await;
            handler.deny_member(&Permissions::VIEW_CHANNEL, user_id).await;
        }
        for role in self.allowed_roles.iter() {
            handler.deny_role(&Permissions::SEND_MESSAGES, role).await;
            handler.deny_role(&Permissions::VIEW_CHANNEL, role).await;
        }
    }

    pub async fn claim(&self, staff: &UserId) {
        let handler = self.get_permissions();
        for role in self.allowed_roles.iter() {
            handler.allow_role(&Permissions::SEND_MESSAGES, role).await;
        }
        self.present_staff.lock().await.insert(*staff);
        self.allow_participants().await;
    }

    pub async fn unclaim(&self, staff: &UserId) {
        self.present_staff.lock().await.remove(staff);
        let handler = self.get_permissions();
        handler.deny_member(&Permissions::SEND_MESSAGES, staff).await;
        handler.deny_member(&Permissions::VIEW_CHANNEL, staff).await;
    }

    pub async fn add_member(&self, member: &UserId) {
        self.present_members.lock().await.insert(*member);
        let handler = self.get_permissions();
        handler.allow_member(&Permissions::VIEW_CHANNEL, member).await;
        handler.allow_member(&Permissions::SEND_MESSAGES, member).await;
    }

    pub async fn remove_member(&self, member: &UserId) {
        self.present_members.lock().await.remove(member);
        let handler = self.get_permissions();
        handler.deny_member(&Permissions::VIEW_CHANNEL, member).await;
        handler.deny_member(&Permissions::SEND_MESSAGES, member).await;
    }

}



use serenity::model::permissions::Permissions;
use serenity::model::channel::GuildChannel;
use serenity::model::id::{ChannelId, RoleId, UserId};
use serenity::builder::{CreateChannel, GetMessages};
use serenity::model::prelude::*;
use serenity::prelude::*;
use futures::stream::StreamExt;
use uuid::Uuid;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::process::Command;

use crate::utility::*;
use crate::databases::*;


#[cfg(feature = "tickets")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TicketType {
    Muted,
    Discussion,
    Question,
    BugReport,
    UserReport,
    StaffReport,
}

#[cfg(feature = "tickets")]
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

#[cfg(feature = "tickets")]
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

#[cfg(feature = "tickets")]
pub struct Ticket {
    pub channel: GuildChannel,
    pub ticket_type: TicketType,
    pub resolver: Resolver,
    pub uuid: Uuid,

    pub pinged_staff: bool,
    pub present_members: Arc<Mutex<HashSet<UserId>>>,
    pub present_staff: Arc<Mutex<HashSet<UserId>>>,
    pub allowed_roles: Vec<RoleId>,
}

#[cfg(feature = "tickets")]
impl Debug for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ticket_type: String = self.ticket_type.into();
        write!(f, "Ticket ({}): {}", ticket_type, self.channel.name)
    }
}

#[cfg(feature = "tickets")]
pub struct TicketHandler {
    tickets: Arc<Mutex<HashMap<String, Arc<Ticket>>>>,
}

#[cfg(feature = "tickets")]
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
        let channels = resolver.guild_channels().await;

        if let Some(channels) = channels {

            // get the ticket category
            let ticket_category = ConfigDB::get_instance().lock().await
                .get("category_tickets").await.unwrap().to_string();

            // recover all tickets
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
                    let tickets = Arc::clone(&self.tickets);
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
        let logstr = &format!("Creating new ticket '{}'", target.name);
        #[cfg(feature = "debug")]
        Logger::info_long("Start", logstr);

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
            let channel = guild.create_channel(resolver, builder).await;

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
                    uuid: Uuid::new_v4(),

                    pinged_staff: false,
                    present_members: present_members,
                    present_staff: Arc::new(Mutex::new(HashSet::new())),
                    allowed_roles: allowed_roles.clone(),
                });

                ticket.allow_participants().await;
                self.tickets.lock().await.insert(ticket.channel.id.to_string(), Arc::clone(&ticket));

                #[cfg(feature = "debug")]
                Logger::info_long("End", logstr);

                return Ok(ticket);
            }
        }

        Err("Failed to create ticket".into())
    }

    pub async fn close_ticket(&self, channel: &ChannelId) {

        let ticket = self.get_ticket(channel).await;
        if let Some(ticket) = ticket {

            #[cfg(feature = "debug")]
            let logstr = &format!("Closing ticket '{}'", ticket.channel.name);
            #[cfg(feature = "debug")]
            Logger::info_long("Start", logstr);

            ticket.deny_all().await;
            let _ = ticket.channel.delete(&ticket.resolver).await;

            #[cfg(feature = "debug")]
            Logger::info_long("End", logstr);
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

#[cfg(feature = "tickets")]
impl Ticket {

    const ACCESS_PERM: Permissions = Permissions::SEND_MESSAGES.union(Permissions::VIEW_CHANNEL);

    pub fn get_permissions<'a>(&'a self) -> PermissionHandler<'a> {
        PermissionHandler::new(&self.resolver, &self.channel)
    }

    pub async fn parse_ticket(resolver: &Resolver, channel: &GuildChannel) -> Result<Ticket> {

        #[cfg(feature = "debug")]
        let logstr = &format!("Parsing ticket '{}'", channel.name);
        #[cfg(feature = "debug")]
        Logger::info_long("Start", logstr);

        let last_message = channel.last_message_id;
        if let Some(last_message) = last_message {

            // get all messages in the channel
            let builder = GetMessages::new().around(last_message).limit(255);
            let messages = channel.messages(&resolver, builder).await;

            if let Ok(messages) = &messages {

                let bot_id = Arc::new(ConfigDB::get_instance().lock().await
                    .get("bot_id").await.unwrap().to_string());
                let regex = Arc::new(RegexManager::get_id_regex());

                // get all present staff and members in the ticket
                let present_staff = Arc::new(Mutex::new(HashSet::new()));
                let present_members = Arc::new(Mutex::new(HashSet::new()));
                let message_iter = futures::stream::iter(messages.iter().rev())
                    .for_each_concurrent(None, |message: &Message| {
                        let present_staff = Arc::clone(&present_staff);
                        let present_members = Arc::clone(&present_members);
                        let bot_id = Arc::clone(&bot_id);
                        let regex = Arc::clone(&regex);
                        async move {
                            let author = &message.author;
                            if !author.bot {
                                present_members.lock().await.insert(author.id);
                            } else {
                                if author.id.to_string() == *bot_id {
                                    if message.embeds.len() > 0 {
                                        if let Some(description) = &message.embeds[0].description {
                                            let splits = &description.split_whitespace().collect::<Vec<&str>>();
                                            let user_id = regex.find(splits.last().unwrap())
                                                .map(|hit| UserId::from(hit.as_str().parse::<u64>().unwrap()))
                                                .unwrap_or(UserId::from(1));
                                            match splits[0] {
                                                "Added" => present_members.lock().await.insert(user_id),
                                                "Removed" => present_members.lock().await.remove(&user_id),
                                                "Claimed" => present_staff.lock().await.insert(user_id),
                                                "Unclaimed" => present_staff.lock().await.remove(&user_id),
                                                _ => { false }
                                            };
                                        }
                                    }
                                }
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
                    uuid: Uuid::new_v4(),

                    pinged_staff: false,
                    present_members: present_members,
                    present_staff: present_staff,
                    allowed_roles: allowed_roles.clone(),
                };

                // setup permissions
                let handler = ticket.get_permissions();
                handler.deny_role(&Permissions::SEND_MESSAGES, allowed_roles).await;
                ticket.allow_participants().await;

                #[cfg(feature = "debug")]
                Logger::info_long("End", logstr);

                return Ok(ticket);
            }
        }
        Err("Failed to parse ticket".into())
    }

    pub async fn allow_participants(&self) {
        let handler = self.get_permissions();
        for user_id in self.present_members.lock().await.iter() {
            handler.allow_member(&Ticket::ACCESS_PERM, user_id).await;
        }
        for user_id in self.present_staff.lock().await.iter() {
            handler.allow_member(&Ticket::ACCESS_PERM, user_id).await;
        }
        for role_id in self.allowed_roles.iter() {
            handler.allow_role(&Ticket::ACCESS_PERM, role_id).await;
        }
    }

    pub async fn deny_all(&self) {
        let handler = self.get_permissions();
        for user_id in self.present_members.lock().await.iter() {
            handler.deny_member(&Ticket::ACCESS_PERM, user_id).await;
        }
        for user_id in self.present_staff.lock().await.iter() {
            handler.deny_member(&Ticket::ACCESS_PERM, user_id).await;
        }
        #[cfg(feature = "debug")]
        Logger::warn("Denying roles when closing a Ticket is currently disabled");
        // for role_id in self.allowed_roles.iter() {
        //     handler.deny_role(&Ticket::ACCESS_PERM, role_id).await;
        // }
    }

    pub async fn add_staff(&self, staff: &UserId) {
        let handler = self.get_permissions();
        for role in self.allowed_roles.iter() {
            handler.deny_role(&Permissions::SEND_MESSAGES, role).await;
        }
        self.present_staff.lock().await.insert(*staff);
        self.allow_participants().await;
    }

    pub async fn remove_staff(&self, staff: &UserId) {
        self.present_staff.lock().await.remove(staff);
        let handler = self.get_permissions();
        handler.deny_member(&Ticket::ACCESS_PERM, staff).await;
    }

    pub async fn add_member(&self, member: &UserId) {
        self.present_members.lock().await.insert(*member);
        let handler = self.get_permissions();
        handler.allow_member(&Ticket::ACCESS_PERM, member).await;
    }

    pub async fn remove_member(&self, member: &UserId) {
        self.present_members.lock().await.remove(member);
        let handler = self.get_permissions();
        handler.deny_member(&Ticket::ACCESS_PERM, member).await;
    }

    pub async fn transcribe(&self) {

        #[cfg(feature = "debug")]
        let logstr = &format!("Transcribing ticket '{}'", self.channel.name);
        #[cfg(feature = "debug")]
        Logger::info_long("Start", logstr);

        let token = ConfigDB::get_instance().lock().await
            .get("token").await.unwrap().to_string();

        if cfg!(target_os = "linux") {
            let process = Command::new("python3")
                .arg("src/commands/tickets/transcribe.py")
                .arg(token)
                .arg(self.channel.id.to_string())
                .arg(self.channel.guild_id.to_string())
                .arg(self.uuid.to_string())
                .output()
                .expect("Failed to transcribe ticket");

            let err = String::from_utf8_lossy(&process.stderr);
            if !err.is_empty() {
                #[cfg(feature = "debug")]
                Logger::err_long("Failed to transcribe ticket", &err);
            }
        } else {
            #[cfg(feature = "debug")]
            Logger::warn("Transcription is only supported on linux");
        }

        #[cfg(feature = "debug")]
        Logger::info_long("End", logstr);
    }
}


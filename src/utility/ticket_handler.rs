
use serenity::model::permissions::Permissions;
use serenity::model::channel::GuildChannel;
use serenity::model::id::{ChannelId, RoleId, UserId};
use serenity::builder::{CreateChannel, GetMessages};
use serenity::model::prelude::*;
use serenity::prelude::*;
use futures::stream::StreamExt;
use uuid::Uuid;
use once_cell::sync::Lazy;

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::process::Command;
use std::sync::RwLock;

use crate::utility::*;
use crate::databases::*;
use crate::impl_singleton;


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
    tickets: Arc<RwLock<HashMap<String, Arc<Ticket>>>>,
}

#[cfg(feature = "tickets")]
impl_singleton!(TicketHandler);

#[cfg(feature = "tickets")]
impl TicketHandler {

    pub fn new() -> Self {
        TicketHandler {
            tickets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn init(&self, resolver: &Resolver) {

        #[cfg(feature = "debug")]
        Logger::info_long("Start", "Initilizing ticket handler");

        // get all ticket channels
        let channels = resolver.guild_channels().await;

        if let Some(channels) = channels {

            // get the ticket category
            let ticket_category = ConfigDB::get_instance()
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
                            tickets.write().expect("Could not get tickets")
                                .insert(channel.id.to_string(), ticket);
                        }
                    }
                });
            parse.await;
        }

        #[cfg(feature = "debug")]
        Logger::info("Hooking ticket selector");

        let channel_id: ChannelId = ConfigDB::get_instance()
            .get("channel_tickets").await.unwrap().into();
        let channel = resolver.resolve_guild_channel(channel_id).await;

        if let Some(channel) = channel {
            if let Some(last_message_id) = channel.last_message_id {

                // fetch last message
                let message = resolver.resolve_message(channel.id, last_message_id).await;
                if let Some(message) = message {

                    // hook selector
                    if message.author.bot {
                        spawn(hook_ticket_selector(resolver.clone(), message)).await;
                    }
                }
            }
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
            let ticket_category = ConfigDB::get_instance()
                .get("category_tickets").await.unwrap().to_string();
            let ticket_category = ticket_category.parse::<u64>().unwrap();

            // create new channel
            let builder = CreateChannel::new(resolver.resolve_name(target))
                .category(ChannelId::from(ticket_category))
                .topic(ticket_type);
            let channel = guild.create_channel(resolver, builder).await;

            if let Ok(channel) = channel {

                // figure out allowed roles
                let mut allowed_roles = vec!["Administrator", "Head Moderator", "Moderator", "Trial Moderator"];
                match ticket_type {
                    TicketType::StaffReport => {
                        let _ = allowed_roles.pop().unwrap();
                        let _ = allowed_roles.pop().unwrap();
                    },
                    TicketType::UserReport => {
                        let _ = allowed_roles.pop().unwrap();
                    },
                    _ => {},
                };

                let allowed_roles = resolver.resolve_role(allowed_roles).await.unwrap().iter()
                    .map(|role| role.id)
                    .collect::<Vec<RoleId>>();

                let pings = format!("<@{}>", target.id);
                let pings = format!("{pings} {}",
                    match ticket_type {
                        TicketType::Muted | TicketType::Discussion => "".to_string(),
                        _ => allowed_roles.iter().map(|role| format!("<@&{}>", role)).collect::<Vec<String>>().join(" "),
                    });

                let support_response = "Support will be with you shortly. It should not take longer than 10 minutes.";
                let discuss_response = "If you **do not** respond within **2 hours**, this ticket will be closed and **appropriate action** will be taken.";

                let introduction_message = match ticket_type {
                    TicketType::Muted       => format!("A staff member created this **muted ticket** with you to discuss your warnings. {}", discuss_response),
                    TicketType::Discussion  => format!("A staff member created this **discussion ticket** with you to discuss a situation you were involved in. {}", discuss_response),
                    TicketType::StaffReport => format!("{} Please provide the ID of the staff member you are reporting as well as any photo evidence or channel links relevant to this report.", support_response),
                    TicketType::UserReport  => format!("{} Please provide the ID of the user you are reporting as well as any photo evidence or channel links relevant to this report.", support_response),
                    TicketType::BugReport   => format!("{} Please provide photo evidence or channel links of the bug you are reporting.", support_response),
                    TicketType::Question    => format!("{} Ask any server-related questions and a staff member will be able to help you out.", support_response),
                };
                let embed = MessageManager::create_embed(|embed| {
                    embed.description(introduction_message)
                }).await;

                let _ = channel.send_message(resolver, pings.to_message()).await;
                let _ = channel.send_message(resolver, embed.to_message()).await;
                let present_members = Arc::new(Mutex::new(HashSet::new()));
                present_members.lock().await.insert(target.id);

                // create ticket
                let ticket = Arc::new(Ticket {
                    channel: channel.clone(),
                    ticket_type: ticket_type,
                    resolver: resolver.clone(),
                    uuid: Uuid::new_v4(),

                    pinged_staff: false,
                    present_members: present_members,
                    present_staff: Arc::new(Mutex::new(HashSet::new())),
                    allowed_roles: allowed_roles.clone(),
                });

                ticket.allow_participants().await;
                self.tickets.write().expect("Could not get tickets")
                    .insert(channel.id.to_string(), Arc::clone(&ticket));

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
        let tickets = self.tickets.read().expect("Could not get tickets");
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

    pub fn get_permissions<'a>(&'a self) -> PermissionHandler<'a> {
        PermissionHandler::new(&self.resolver, &self.channel)
    }

    async fn append_user(resolver: &Resolver, user: &User, members: &mut HashSet<UserId>, staff: &mut HashSet<UserId>) {
        match is_trial(resolver, user).await {
            true  => staff.insert(user.id),
            false => members.insert(user.id),
        };
    }

    pub async fn parse_ticket(resolver: &Resolver, channel: &GuildChannel) -> Result<Ticket> {

        #[cfg(feature = "debug")]
        let logstr = &format!("Parsing ticket '{}'", channel.name);
        #[cfg(feature = "debug")]
        Logger::info_long("Start", logstr);

        let last_message = channel.last_message_id;
        if let Some(last_message) = last_message {

            // get all messages in the channel
            let builder = GetMessages::new().before(last_message).limit(255);
            let messages = channel.messages(&resolver, builder).await;

            if let Ok(messages) = &messages {

                let bot_id = &ConfigDB::get_instance()
                    .get("bot_id").await.unwrap().to_string();
                let regex = Arc::new(RegexManager::get_id_regex());

                // get all present staff and members in the ticket
                let mut present_staff = HashSet::new();
                let mut present_members = HashSet::new();

                // parse all messages
                for message in messages.into_iter().rev() {
                    let author = &message.author;

                    // parse messages
                    if !author.bot {

                        Ticket::append_user(resolver, author, &mut present_members, &mut present_staff).await;

                    } else {

                        // only parse embeds from Kalopsian
                        if author.id.to_string() != *bot_id {
                            continue;
                        }

                        // only parse messages with embeds
                        if message.embeds.len() == 0 {
                            continue;
                        }

                        // parse embeds
                        if let Some(description) = &message.embeds[0].description {
                            let splits = &description.split_whitespace().collect::<Vec<&str>>();
                            let user_id = regex.find(splits.last().unwrap())
                                .map(|hit| UserId::from(hit.as_str().parse::<u64>().unwrap()))
                                .unwrap_or(UserId::from(1));
                            let user = &resolver.resolve_user(user_id).await;
                            if let Some(user) = user {
                                match splits[0] {
                                    "Added" | "Removed" | "Claimed" | "Unclaimed" =>
                                        Ticket::append_user(resolver, user, &mut present_members, &mut present_staff).await,
                                    _ => {}
                                };
                            };
                        }
                    }
                }

                // target should always be present in the ticket
                let first_message = messages.last().unwrap();
                let first_message = &MessageManager::new(resolver.clone(), first_message.clone()).await;
                let mentions = first_message.get_mentions().await;

                // when the ticket is long, the first message is not fetched
                if mentions.len() > 0 {
                    present_members.insert(mentions[0]);
                }

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
                    present_members: Arc::new(Mutex::new(present_members)),
                    present_staff:   Arc::new(Mutex::new(present_staff)),
                    allowed_roles: allowed_roles.clone(),
                };

                // setup permissions
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

        // allow members
        handler.allow_member(
            vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
            &self.present_members.lock().await.iter().collect::<Vec<_>>()
        ).await;

        // allow staff
        handler.allow_member(
            vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
            &self.present_staff.lock().await.iter().collect::<Vec<_>>()
        ).await;

        // setup roles
        match self.present_staff.lock().await.len() > 0 {
            true  => handler.role(Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES, &self.allowed_roles).await,
            false => handler.allow_role(
                vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
                &self.allowed_roles).await
        };
    }

    pub async fn deny_all(&self) {
        let handler = self.get_permissions();
        for user_id in self.present_members.lock().await.iter() {
            handler.member(Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES, user_id).await;
        }
        for user_id in self.present_staff.lock().await.iter() {
            handler.member(Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES, user_id).await;
        }
        #[cfg(feature = "debug")]
        Logger::warn("Denying roles when closing a Ticket is currently disabled");
        // for role_id in self.allowed_roles.iter() {
        //     handler.deny_role(&Ticket::ACCESS_PERM, role_id).await;
        // }
    }

    pub async fn add_staff(&self, staff: &UserId) {
        let handler = self.get_permissions();
        let mut staff_lock = self.present_staff.lock().await;
        staff_lock.insert(*staff);

        // deny all staff to send, when the ticket is freshly claimed
        if staff_lock.len() == 1 {
            handler.role(Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES, &self.allowed_roles).await;
        }

        // grant newly added staff access
        handler.allow_member(
            vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
            staff
        ).await;
    }

    pub async fn remove_staff(&self, staff: &UserId) {
        self.present_staff.lock().await.remove(staff);
        let handler = self.get_permissions();
        handler.deny_member(
            vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
            staff
        ).await;
    }

    pub async fn add_member(&self, member: &UserId) {
        self.present_members.lock().await.insert(*member);
        let handler = self.get_permissions();
        handler.allow_member(
            vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
            member
        ).await;
    }

    pub async fn remove_member(&self, member: &UserId) {
        self.present_members.lock().await.remove(member);
        let handler = self.get_permissions();
        handler.deny_member(
            vec![Permissions::VIEW_CHANNEL, Permissions::SEND_MESSAGES],
            member
        ).await;
    }

    pub async fn transcribe(&self) {

        #[cfg(feature = "debug")]
        let logstr = &format!("Transcribing ticket '{}'", self.channel.name);
        #[cfg(feature = "debug")]
        Logger::info_long("Start", logstr);

        let token = ConfigDB::get_instance()
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


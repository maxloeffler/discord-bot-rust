
use serenity::model::channel::Message;
use serenity::all::{ChannelId, GuildId, Member, User, RoleId, Context};
use serenity::builder::{CreateEmbed, CreateMessage, CreateEmbedFooter};

use std::collections::HashMap;
use std::str::FromStr;

use crate::utility::database::{Database, DB};
use crate::utility::traits::{ToList, Singleton};


impl ToList<RoleId> for String {
    fn to_list(&self) -> Vec<RoleId> {
        let role = RoleId::from_str(self);
        if role.is_ok() {
            return vec![role.unwrap()];
        }
        Vec::new()
    }
}

impl ToList<RoleId> for Vec<String> {
    fn to_list(&self) -> Vec<RoleId> {
        let mut roles = Vec::new();
        for role in self {
            let role = RoleId::from_str(role);
            if role.is_ok() {
                roles.push(role.unwrap());
            }
        }
        roles
    }
}

trait ToSend {
    fn to_send(&self) -> CreateMessage;
}
impl ToSend for &str {
    fn to_send(&self) -> CreateMessage {
        CreateMessage::default().content(self.to_string())
    }
}
impl ToSend for CreateEmbed {
    fn to_send(&self) -> CreateMessage {
        CreateMessage::default().embed(self.clone())
    }
}
impl ToSend for CreateMessage {
    fn to_send(&self) -> CreateMessage {
        self.clone()
    }
}
impl ToSend for String {
    fn to_send(&self) -> CreateMessage {
        CreateMessage::default().content(self.to_string())
    }
}
impl ToSend for &String {
    fn to_send(&self) -> CreateMessage {
        CreateMessage::default().content(self.to_string())
    }
}

#[derive(Clone)]
pub struct MessageManager {
    ctx: Context,
    raw_message: Message,
    command: Option<String>,
    parameters: HashMap<String, Vec<String>>,
    words: Vec<String>
}

impl MessageManager {

    pub async fn new(ctx: Context, raw_message: Message) -> MessageManager {
        let mut manager = MessageManager {
            ctx,
            raw_message,
            command: None,
            parameters: HashMap::new(),
            words: Vec::new()
        };
        manager.parse_message().await;
        manager
    }

    fn first_word_index(&self) -> usize {
        if self.is_command() { 1 } else { 0 }
    }

    async fn parse_message(&mut self) {
        let mut key = String::new();
        let mut value = Vec::new();

        // Obtain words
        self.words = self.raw_message.content
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();

        // Obtian command
        if self.words.len() > 0 {
            let config = Database::get_instance().lock().await;
            let prefix = config.get(DB::Config, "command_prefix").await.unwrap();
            if self.words[0].starts_with(&prefix) {
                let command = self.words[0].to_string();
                self.command = command.strip_prefix(&prefix).map(|s| s.to_string());
            }
        }

        // Obtain parameters
        if self.words.len() > self.first_word_index() {
            for word in &self.words[1..] {
                if word.starts_with("-") {
                    if key != "" {
                        self.parameters.insert(key, value.clone());
                        value = Vec::new();
                    }
                    key = word.to_string();
                } else {
                    value.push(word.to_string());
                }
            }
            if key != "" && !self.parameters.contains_key(&key) {
                self.parameters.insert(key, value);
            }
        }
    }

    pub fn is_command(&self) -> bool {
        self.command.is_some()
    }

    pub fn get_command(&self) -> Option<String> {
        self.command.clone()
    }

    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    pub fn get_parameter(&self, key: &str) -> String {
        self.parameters.get(key).unwrap().join(" ")
    }

    pub fn payload(&self, starting_from: Option<usize>, excludes: Option<Vec<String>>) -> String {
        let first_word = match starting_from {
            Some(starting_from) => self.first_word_index() + starting_from,
            None => self.first_word_index()
        };
        let words = &self.words[first_word..];
        let excludes = match excludes {
            Some(excludes) => excludes,
            None => Vec::new()
        };
        let mut payload = String::new();
        for word in words {
            if !excludes.contains(word) {
                payload.push_str(word);
                payload.push_str(" ");
            }
        }
        payload.trim().to_string()
    }

    pub async fn delete(&self) {
        let timeout = 250;
        let mut attempts = 5;
        while self.raw_message.delete(&self.ctx).await.is_err() && attempts > 0 {
            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(timeout));
            attempts -= 1;
        }
    }

    pub async fn reply(&self, message: impl ToSend) {
        let channel = self.get_channel();
        let _ = channel.send_message(self.ctx.http.clone(), message.to_send()).await;
    }

    pub async fn create_embed(&self, fn_style: impl FnOnce(CreateEmbed) -> CreateEmbed) -> Result<CreateEmbed, &str> {
        let color_primary = Database::get_instance().lock().await.get(DB::Config, "color_primary").await;
        if color_primary.is_some() {
            let embed = fn_style(CreateEmbed::default());
            let styled_embed = embed.clone()
                .color(color_primary.clone().unwrap().parse::<u32>().unwrap());
            return Ok(styled_embed);
        }
        Err("'color_primary' not configured")
    }

    // ---- Permissions ---- //

    pub async fn has_role(&self, roles: impl ToList<RoleId>) -> bool {
        let member_roles = self.get_roles().await;
        if member_roles.is_some() {
            for role in roles.to_list() {
                if member_roles.clone().unwrap().contains(&role) {
                    return true;
                }
            }
        }
        false
    }

    pub async fn is_admin(&self) -> bool {
        let config = Database::get_instance().lock().await;
        let role_admin_id = config.get(DB::Config, "role_admin_id").await;
        match role_admin_id {
            Some(role) => self.has_role(role).await,
            _ => false
        }
    }

    pub async fn is_headmod(&self) -> bool {
        let config = Database::get_instance().lock().await;
        let role_ids = config.get_multiple(DB::Config, vec!["role_admin_id", "role_headmod_id"]).await;
        match role_ids {
            Some(roles) => self.has_role(roles).await,
            _ => false
        }
    }

    pub async fn is_mod(&self) -> bool {
        let config = Database::get_instance().lock().await;
        let role_ids = config.get_multiple(DB::Config, vec!["role_admin_id", "role_headmod_id", "role_mod_id"]).await;
        match role_ids {
            Some(roles) => self.has_role(roles).await,
            _ => false
        }
    }

    pub async fn is_trial(&self) -> bool {
        let config = Database::get_instance().lock().await;
        let role_ids = config.get_multiple(DB::Config, vec!["role_admin_id", "role_headmod_id", "role_mod_id", "role_trial_id"]).await;
        match role_ids {
            Some(roles) => self.has_role(roles).await,
            _ => false
        }
    }


    // ---- Basics ---- //

    pub fn get_channel(&self) -> ChannelId {
        self.raw_message.channel_id
    }

    pub fn get_guild(&self) -> Option<GuildId> {
        self.raw_message.guild_id
    }

    pub async fn get_member(&self) -> Option<Member> {
        let guild = self.get_guild();
        if guild.is_some() {
            let user_id = self.get_author().id;
            let member = guild.unwrap().member(&self.ctx.http, user_id).await;
            return match member {
                Ok(member) => Some(member),
                Err(..) => None
            };
        }
        None
    }

    pub async fn get_roles(&self) -> Option<Vec<RoleId>> {
        let member = self.get_member().await;
        if member.is_some() {
            return Some(member.unwrap().roles);
        }
        None
    }

    pub fn get_author(&self) -> User {
        self.raw_message.author.clone()
    }
}

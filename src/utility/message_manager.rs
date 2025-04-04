
use serenity::model::prelude::*;
use serenity::builder::{
    CreateEmbed,
    GetMessages
};
use serenity::all::{CacheHttp, Cache, Http};
use nonempty::NonEmpty;
use regex::Regex;

use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;

use crate::utility::*;
use crate::databases::*;


#[derive(Clone)]
pub struct MessageManager {
    resolver: Resolver,
    raw_message: Message,
    prefix: Option<String>,
    command: Option<String>,
    parameters: HashMap<String, String>,
    pub words: Vec<String>
}

impl CacheHttp for MessageManager {
    fn http(&self) -> &Http {
        self.resolver.http()
    }
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(self.resolver.cache())
    }
}

impl AsRef<Http> for MessageManager {
    fn as_ref(&self) -> &Http {
        self.resolver.http()
    }
}

impl MessageManager {

    pub async fn new(resolver: Resolver, message: Message) -> MessageManager {
        let mut manager = MessageManager {
            resolver: resolver,
            raw_message: message,
            prefix: None,
            command: None,
            parameters: HashMap::new(),
            words: Vec::new()
        };
        manager.parse_message().await;
        manager
    }

    pub async fn spoof(&self, content: String) -> MessageManager {
        let mut spoofed_message = self.raw_message.clone();
        spoofed_message.content = content;
        MessageManager::new(self.resolver.clone(), spoofed_message).await
    }

    fn first_word_index(&self) -> usize {
        if self.is_command() { 1 } else { 0 }
    }

    async fn parse_message(&mut self) {
        // Obtain words
        self.words = self.raw_message.content
            .split_whitespace()
            .map(|word| word.to_string())
            .collect();

        // Obtain command
        if self.words.len() > 0 {
            let prefix = ConfigDB::get_instance()
                .get("command_prefix").await.unwrap().to_string();
            if self.words[0].starts_with(&prefix) && self.words[0].len() > 1 {
                let command = self.words[0].to_string();
                self.command = command.strip_prefix(&prefix).map(|s| s.to_string());
                self.prefix = Some(prefix);
            }
        }

        // Only parse parameters if message is a command
        if self.is_command() {

            // Obtain keys
            let keys = self.words.iter()
                .filter(|word| word.starts_with("-"))
                .map(|word| word.to_string())
                .collect::<Vec<String>>();

            // Iterate over keys
            if keys.len() > 0 {
                let split_regex = Regex::new(keys.join("|").as_str());

                if let Ok(regex) = split_regex {
                    let payload = self.raw_message.content.clone();
                    let splits = regex.split(&payload).collect::<Vec<&str>>();
                    splits[1..].into_iter()
                        .enumerate()
                        .for_each(|(i, split)| {
                            let key = keys[i].strip_prefix("-").unwrap().to_string();
                            self.parameters.insert(key, split.trim().to_string());
                        });
                }
            }
        }
    }

    pub fn is_command(&self) -> bool {
        self.command.is_some()
    }

    pub fn get_command(&self) -> Option<String> {
        self.command.clone()
    }

    pub fn get_prefix(&self) -> Option<String> {
        self.prefix.clone()
    }

    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    pub fn get_parameter(&self, key: &str) -> String {
        self.parameters.get(key).unwrap().to_string()
    }

    pub fn payload(&self, starting_from: Option<usize>, excludes: Option<Vec<String>>) -> String {

        // calculate starting index
        let first = match starting_from {
            Some(starting_from) => self.first_word_index() + starting_from,
            None => self.first_word_index()
        };

        // obtain words
        let words = &self.words[first..];
        let payload = match excludes {
            Some(excludes) => {

                // filter words
                let pattern = excludes.into_iter()
                    .map(|exclude| RegexManager::escape(exclude.as_str()))
                    .collect::<Vec<_>>()
                    .join("|");
                let regex = Regex::new(&pattern).unwrap();
                words.iter()
                    .filter(|word| !regex.is_match(word))
                    .map(|word| word.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")

            },
            None => words.join(" ")
        };

        payload.trim().to_string()
    }

    pub fn payload_without_mentions(&self, starting_from: Option<usize>, excludes: Option<Vec<String>>) -> String {
        let mut total_excludes = excludes.unwrap_or(Vec::new());
        let regex_id = RegexManager::get_id_regex();
        total_excludes.push(regex_id.as_str().to_string());
        self.payload(starting_from, Some(total_excludes))
    }

    pub fn get_referenced(&self) -> Box<Message> {
        self.raw_message.referenced_message.clone().unwrap()
    }

    pub fn is_referencing(&self) -> bool {
        self.raw_message.referenced_message.is_some()
    }

    pub async fn delete(&self) {
        let _ = self.raw_message.delete(&self.resolver).await;
    }

    pub async fn reply(&self, message: impl ToMessage) -> Result<Message> {
        let channel = self.get_channel();
        channel.send_message(&self.resolver, message.to_message()).await.map_err(|_| "Failed to send message".to_string())
    }

    pub async fn reply_temporary(&self, message: impl ToMessage) {
        let sent_message = self.reply(message).await;
        if let Ok(message) = sent_message {
            let _ = tokio::time::sleep(Duration::from_secs(4)).await;
            let _ = message.delete(&self.resolver).await;
        }
    }

    pub async fn reply_success(&self) {
        let embed = MessageManager::create_embed(|embed| {
            embed
                .title("✅")
                .description("Success!")
        }).await;
        self.reply_temporary(embed).await;
    }

    pub async fn reply_failure(&self, context: &str) {
        let embed = MessageManager::create_embed(|embed| {
            embed
                .title("❌")
                .description(context)
        }).await;
        self.reply_temporary(embed).await;
    }

    pub async fn create_embed(fn_style: impl FnOnce(CreateEmbed) -> CreateEmbed) -> CreateEmbed {
        let color_primary = ConfigDB::get_instance()
            .get("color_primary").await.unwrap();
        let embed = fn_style(CreateEmbed::default());
        let styled_embed = embed.color(color_primary);
        styled_embed
    }

    pub async fn last_messages(&self, limit: u8) -> Vec<Message> {
        let channel = self.get_channel();
        let builder = GetMessages::new().before(self.raw_message.id).limit(limit);
        let messages = channel.messages(&self.resolver, builder).await;
        match messages {
            Ok(messages) => messages,
            Err(_) => Vec::new()
        }
    }

    pub fn get_interaction_helper(&self) -> InteractionHelper {
        InteractionHelper::new(self.get_channel(), self.get_resolver())
    }

    pub async fn react(&self, reaction: &str) {
        let _ = self.raw_message.react(&self.resolver, ReactionType::Unicode(reaction.to_string())).await;
    }

    // ---- Basics ---- //

    pub fn get_channel(&self) -> ChannelId {
        self.raw_message.channel_id
    }

    pub fn get_guild(&self) -> Option<GuildId> {
        self.raw_message.guild_id
    }

    pub fn get_author(&self) -> &User {
        &self.raw_message.author
    }

    pub async fn get_mentions(&self) -> Vec<UserId> {
        let mut mentions = Vec::new();

        let id_regex = RegexManager::get_id_regex();
        for word in &self.words {
            let find = id_regex.find(word);
            if find.is_some() {
                let id = find.unwrap().as_str().parse::<u64>();
                match id {
                    Ok(id) => mentions.push(UserId::from(id)),
                    Err(_) => {}
                };
            }
        }
        mentions
    }

    pub async fn get_mentioned_roles(&self) -> Vec<RoleId> {
        let mut mentions = Vec::new();

        let role_regex = RegexManager::get_role_regex();
        let id_regex = RegexManager::get_id_regex();
        for word in &self.words {
            let find = role_regex.find(word);
            if find.is_some() {
                let find = id_regex.find(find.unwrap().as_str());
                let id = find.unwrap().as_str().parse::<u64>();
                match id {
                    Ok(id) => mentions.push(RoleId::from(id)),
                    Err(_) => {}
                };
            }
        }
        mentions
    }

    pub async fn get_attachments(&self) -> &Vec<Attachment> {
        &self.raw_message.attachments
    }

    pub fn get_timestamp(&self) -> i64 {
        self.raw_message.timestamp.timestamp()
    }

    pub fn get_log_builder(&self) -> LogBuilder {
        LogBuilder::new(self)
    }

    // ---- Forwards to Resolver ---- //

    pub fn get_resolver(&self) -> &Resolver {
        &self.resolver
    }

    pub async fn resolve_role(&self, role_name: impl ToList<&str>) -> Option<Vec<Role>> {
        self.resolver.resolve_role(role_name).await
    }

    pub async fn resolve_member(&self) -> Option<Member> {
        self.resolver.resolve_member(self.get_author()).await
    }

    pub async fn resolve_guild_channel(&self) -> Option<GuildChannel> {
        self.resolver.resolve_guild_channel(self.get_channel()).await
    }

    pub fn resolve_name(&self) -> String {
        self.resolver.resolve_name(self.get_author())
    }

    pub async fn has_role(&self, roles: impl ToList<RoleId>) -> bool {
        self.resolver.has_role(self.get_author(), roles).await
    }

    pub async fn is_admin(&self) -> bool {
        self.resolver.is_admin(self.get_author()).await
    }

    pub async fn is_headmod(&self) -> bool {
        self.resolver.is_headmod(self.get_author()).await
    }

    pub async fn is_mod(&self) -> bool {
        self.resolver.is_mod(self.get_author()).await
    }

    pub async fn is_trial(&self) -> bool {
        self.resolver.is_trial(self.get_author()).await
    }

}

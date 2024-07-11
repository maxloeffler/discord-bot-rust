
use serenity::model::channel::Message;
use serenity::all::{ChannelId, User, Context};
use tokio::sync::Mutex;

use std::sync::Arc;
use std::collections::HashMap;

use crate::utility::database::Database;


#[derive(Clone)]
pub struct MessageManager {
    config: Arc<Mutex<Database>>,
    ctx: Context,
    raw_message: Message,
    command: Option<String>,
    parameters: HashMap<String, Vec<String>>,
    words: Vec<String>
}

impl MessageManager {

    pub async fn new(config: Arc<Mutex<Database>>, ctx: Context, raw_message: Message) -> MessageManager {
        let mut manager = MessageManager {
            config,
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
            let prefix = self.config.lock().await.get("command_prefix").await;
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
        payload
    }

    pub async fn reply(&self, message: &str) {
        let channel = self.get_channel();
        channel.say(&self.ctx, message).await.unwrap();
    }

    // ---- Forwards ---- //

    pub fn get_channel(&self) -> ChannelId {
        self.raw_message.channel_id
    }

    pub fn get_author(&self) -> User {
        self.raw_message.author.clone()
    }
}

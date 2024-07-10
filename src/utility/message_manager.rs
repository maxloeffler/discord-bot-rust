
use serenity::model::channel::Message;
use serenity::all::ChannelId;
use tokio::sync::Mutex;

use std::sync::Arc;
use std::collections::HashMap;

use crate::utility::database::Database;


pub struct MessageManager {
    config: Arc<Mutex<Database>>,
    raw_message: Message,
    command: Option<String>,
    parameters: HashMap<String, Vec<String>>,
    words: Vec<String>
}

impl MessageManager {

    pub async fn new(config: Arc<Mutex<Database>>, raw_message: Message) -> MessageManager {
        let mut manager = MessageManager {
            config,
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
            if self.words[0].starts_with(&self.config.lock().await.get("command_prefix").await) {
                self.command = Some(self.words[0].to_string());
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

    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    pub fn get_parameter(&self, key: &str) -> String {
        self.parameters.get(key).unwrap().join(" ")
    }

    pub fn payload(&self, starting_from: Option<i32>, excludes: Option<Vec<String>>) -> String {
        let first_word = self.first_word_index();
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

    // ---- Forwards ---- //

    pub fn get_channel(&self) -> ChannelId {
        self.raw_message.channel_id
    }

    pub fn get_author(&self) -> User{
        self.raw_message.author
    }
}

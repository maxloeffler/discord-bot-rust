
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::all::ComponentInteractionDataKind::StringSelect;
use serenity::model::application::ButtonStyle;
use serenity::builder::{
    CreateEmbed,
    CreateMessage,
    CreateButton,
    CreateInteractionResponse,
    CreateSelectMenu,
    CreateSelectMenuKind,
    CreateSelectMenuOption,
    GetMessages
};
use nonempty::NonEmpty;
use regex::Regex;

use std::collections::HashMap;
use std::time::Duration;

use crate::databases::*;
use crate::utility::traits::{Singleton, ToList, ToMessage};
use crate::utility::mixed::{BoxedFuture, RegexManager};
use crate::utility::resolver::Resolver;


#[derive(Clone)]
pub struct MessageManager {
    ctx: Context,
    raw_message: Message,
    prefix: Option<String>,
    command: Option<String>,
    parameters: HashMap<String, Vec<String>>,
    words: Vec<String>
}

impl MessageManager {

    pub async fn new(ctx: Context, raw_message: Message) -> MessageManager {
        let mut manager = MessageManager {
            ctx,
            raw_message,
            prefix: None,
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
            let prefix = ConfigDB::get_instance().lock().await.get("command_prefix").await.unwrap();
            if self.words[0].starts_with(&prefix) {
                let command = self.words[0].to_string();
                self.command = command.strip_prefix(&prefix).map(|s| s.to_string());
                self.prefix = Some(prefix);
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

    pub fn get_prefix(&self) -> Option<String> {
        self.prefix.clone()
    }

    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    pub fn get_parameter(&self, key: &str) -> String {
        self.parameters.get(key).unwrap().join(" ")
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
                let pattern = excludes.join("|");
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

    pub async fn payload_without_mentions(&self, starting_from: Option<usize>, excludes: Option<Vec<String>>) -> String {
        let mut total_excludes = excludes.unwrap_or(Vec::new());
        let regex_id = RegexManager::get_instance().lock().await
            .get_id_regex();
        total_excludes.push(regex_id.as_str().to_string());
        self.payload(starting_from, Some(total_excludes))
    }

    pub async fn delete(&self) {
        let timeout = 250;
        let mut attempts = 5;
        while self.raw_message.delete(&self.ctx).await.is_err() && attempts > 0 {
            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(timeout));
            attempts -= 1;
        }
    }

    pub async fn reply(&self, message: impl ToMessage) {
        let channel = self.get_channel();
        let _ = channel.send_message(self.ctx.http.clone(), message.to_message()).await;
    }

    pub async fn reply_success(&self) {

        // prepare message
        let embed = MessageManager::create_embed(|embed| {
            embed
                .title("✅")
                .description("Success!")
        }).await;
        let message = CreateMessage::new().embed(embed);

        // send message
        let channel = self.get_channel();
        let sent_message = channel.send_message(&self.ctx.http, message).await;

        // delete message
        if let Ok(message) = sent_message {
            let _ = tokio::time::sleep(Duration::from_secs(3)).await;
            let _ = message.delete(&self.ctx).await;
        }
    }

    pub async fn reply_failure(&self, context: &str) {

        // prepare message
        let embed = MessageManager::create_embed(|embed| {
            embed
                .title("❌")
                .description(context)
        }).await;
        let message = CreateMessage::new().embed(embed);

        // send message
        let channel = self.get_channel();
        let sent_message = channel.send_message(&self.ctx.http, message).await;

        // delete message
        if let Ok(message) = sent_message {
            let _ = tokio::time::sleep(Duration::from_secs(3)).await;
            let _ = message.delete(&self.ctx).await;
        }
    }

    pub async fn create_embed(fn_style: impl FnOnce(CreateEmbed) -> CreateEmbed) -> CreateEmbed {
        let color_primary = ConfigDB::get_instance().lock().await.get("color_primary").await.unwrap();
        let embed = fn_style(CreateEmbed::default());
        let styled_embed = embed.clone()
            .color(color_primary.clone().parse::<u32>().unwrap());
        styled_embed
    }

    pub async fn get_last_messages(&self, limit: u8) -> Vec<Message> {
        let channel = self.get_channel();
        let builder = GetMessages::new().around(self.raw_message.id).limit(limit);
        let messages = channel.messages(&self.ctx.http, builder).await;
        match messages {
            Ok(messages) => messages,
            Err(_) => Vec::new()
        }
    }


    // ---- Move to interaction_manager at some point ---- //

    pub async fn create_choice_interaction<'a>(&self,
                                     message: impl ToMessage,
                                     yes_callback: BoxedFuture<'a, ()>,
                                     no_callback:  BoxedFuture<'a, ()>) {

        // prepare message
        let yes_button = CreateButton::new("Yes")
            .label("Yes")
            .style(ButtonStyle::Primary);
        let no_button  = CreateButton::new("No")
            .label("No")
            .style(ButtonStyle::Secondary);
        let message = message.to_message().button(yes_button).button(no_button);

        // send message
        let sent_message = self.get_channel()
            .send_message(&self.ctx.http.clone(), message).await.unwrap();

        // await interaction
        let interaction = sent_message
            .await_component_interaction(&self.ctx.shard)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if interaction.is_some() {

            match interaction.clone().unwrap().data.custom_id.as_str() {
                "Yes" => yes_callback.await,
                "No"  => no_callback.await,
                _ => {}
            };

            // end interaction
            let _ = interaction.unwrap().create_response(&self.ctx.http,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.ctx).await;

        }
    }

    pub async fn create_dropdown_interaction<'a>(&self,
                                        message: impl ToMessage,
                                        options: Vec<CreateSelectMenuOption>,
                                        callback: impl FnOnce(String) -> BoxedFuture<'a, ()>) {

        // prepare message
        let message = message.to_message().select_menu(
            CreateSelectMenu::new("select_menu", CreateSelectMenuKind::String {
                options: options
            })
            .placeholder("Select an option")
        );

        // send message
        let sent_message = self.get_channel()
            .send_message(&self.ctx.http.clone(), message).await.unwrap();

        // await interaction
        let interaction = sent_message
            .await_component_interaction(&self.ctx.shard)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if interaction.is_some() {

            let data = interaction.clone().unwrap().data.kind;
            match data {
                StringSelect{values} => {
                    callback(values[0].clone()).await;
                }
                _ => {}
            }

            // end interaction
            let _ = interaction.unwrap().create_response(&self.ctx.http,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.ctx).await;

        }
    }

    pub async fn create_user_dropdown_interaction<'a>(&self,
                                        message: impl ToMessage,
                                        users: Vec<User>,
                                        callback: impl FnOnce(User) -> BoxedFuture<'a, ()>) {

        // prepare message
        let message = message.to_message().select_menu(
            CreateSelectMenu::new("user_select_menu", CreateSelectMenuKind::String {
                options: users.iter().map(|user| {
                    CreateSelectMenuOption::new(user.name.clone(), user.id.to_string())
                        .description(&user.id.to_string())
                }).collect()
            })
            .placeholder("Select a user")
        );

        // send message
        let sent_message = self.get_channel()
            .send_message(&self.ctx.http.clone(), message).await.unwrap();

        // await interaction
        let interaction = sent_message
            .await_component_interaction(&self.ctx.shard)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if interaction.is_some() {

            let data = interaction.clone().unwrap().data.kind;
            match data {
                StringSelect{values} => {
                    let id = values[0].clone().parse::<u64>().unwrap();
                    let user = Resolver::get_instance().lock().await
                        .get_user(self.ctx.clone(), UserId::from(id)).await;
                    if user.is_some() {
                        callback(user.unwrap()).await;
                    }
                }
                _ => {}
            }

            // end interaction
            let _ = interaction.unwrap().create_response(&self.ctx.http,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.ctx).await;

        }
    }

    // ---- Basics ---- //

    pub fn get_channel(&self) -> ChannelId {
        self.raw_message.channel_id
    }

    pub fn get_guild(&self) -> Option<GuildId> {
        self.raw_message.guild_id
    }

    pub fn get_author(&self) -> User {
        self.raw_message.author.clone()
    }

    pub async fn get_mentions(&self) -> NonEmpty<User> {
        let author = self.get_author();
        let mut mentions = NonEmpty::new(author);

        let id_regex = RegexManager::get_instance().lock().await.get_id_regex();
        for word in &self.words {
            let find = id_regex.find(word);
            if find.is_some() {
                let id = find.unwrap().as_str().parse::<u64>();
                match id {
                    Ok(id) => {
                        let user = self.ctx.http.get_user(id.into()).await;
                        match user {
                            Ok(user) => mentions.push(user),
                            Err(_) => {}
                        }
                    },
                    Err(_) => {}
                };
            }
        }
        mentions
    }

    // ---- Forwards to Resolver ---- //

    pub async fn get_member(&self) -> Option<Member> {
        let user = self.get_author();
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.get_member(self.ctx.clone(), guild_id, user).await
    }

    pub async fn has_role(&self, roles: impl ToList<RoleId>) -> bool {
        let user = self.get_author();
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.has_role(self.ctx.clone(), guild_id, user, roles).await
    }

    pub async fn get_roles(&self) -> Option<Vec<RoleId>> {
        let user = self.get_author();
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.get_roles(self.ctx.clone(), guild_id, user).await
    }

    pub async fn is_admin(&self) -> bool {
        let user = self.get_author();
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.is_admin(self.ctx.clone(), guild_id, user).await
    }

    pub async fn is_headmod(&self) -> bool {
        let user = self.get_author();
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.is_headmod(self.ctx.clone(), guild_id, user).await
    }

    pub async fn is_mod(&self) -> bool {
        let user = self.get_author();
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.is_mod(self.ctx.clone(), guild_id, user).await
    }

    pub async fn is_trial(&self, user: Option<User>) -> bool {
        let user = user.unwrap_or(self.get_author());
        let guild_id = self.get_guild();
        Resolver::get_instance().lock().await.is_trial(self.ctx.clone(), guild_id, user).await
    }

}

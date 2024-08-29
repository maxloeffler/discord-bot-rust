
use serenity::model::prelude::*;
use serenity::builder::{CreateEmbed, CreateMessage, CreateButton};
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use nonempty::NonEmpty;

use std::sync::Arc;
use std::str::FromStr;

#[cfg(feature = "auto_moderation")]
use crate::utility::auto_moder::AutoModerator;
#[cfg(feature = "tickets")]
use crate::utility::ticket_handler::TicketHandler;
use crate::utility::message_manager::MessageManager;
use crate::utility::mixed::{string_distance, BoxedFuture, Result};
use crate::databases::*;


pub trait Singleton: Sized {
    fn get_instance() -> &'static Self;
    fn new() -> Self;
}

#[macro_export]
macro_rules! impl_singleton {
    ($t:ty) => {
        impl Singleton for $t {
            fn get_instance() -> &'static Self {
                static INSTANCE: Lazy<Arc<$t>> = Lazy::new(|| Arc::new(<$t>::new()));
                &INSTANCE
            }

            fn new() -> Self {
                <$t>::new()
            }
        }
    };
}


pub trait ToList<T: ?Sized> {
    fn to_list(&self) -> Vec<T> where T: Clone;
}

impl<T> ToList<T> for T {
    fn to_list(&self) -> Vec<T> where T: Clone {
        vec![self.clone()]
    }
}

impl<T> ToList<T> for Vec<T> {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.clone()
    }
}

impl<T> ToList<T> for [T] {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.iter().map(|s| s.clone()).collect()
    }
}

impl<T> ToList<T> for &T {
    fn to_list(&self) -> Vec<T> where T: Clone {
        vec![(*self).clone()]
    }
}

impl<T> ToList<T> for Vec<&T> {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.iter().map(|s| (*s).clone()).collect()
    }
}

impl<T> ToList<T> for &[T] {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.iter().map(|s| s.clone()).collect()
    }
}


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

impl ToList<RoleId> for &str {
    fn to_list(&self) -> Vec<RoleId> {
        let role = RoleId::from_str(self);
        if role.is_ok() {
            return vec![role.unwrap()];
        }
        Vec::new()
    }
}

impl ToList<RoleId> for Vec<DBEntry> {
    fn to_list(&self) -> Vec<RoleId> {
        let mut roles = Vec::new();
        for role in self {
            let role = RoleId::from_str(&role.to_string());
            if role.is_ok() {
                roles.push(role.unwrap());
            }
        }
        roles
    }
}

impl ToList<RoleId> for DBEntry {
    fn to_list(&self) -> Vec<RoleId> {
        let role = RoleId::from_str(&self.to_string());
        if role.is_ok() {
            return vec![role.unwrap()];
        }
        Vec::new()
    }
}

impl ToList<RoleId> for Vec<Role> {
    fn to_list(&self) -> Vec<RoleId> {
        self.iter().map(|role| role.id).collect()
    }
}

impl ToList<RoleId> for &Role {
    fn to_list(&self) -> Vec<RoleId> {
        vec![self.id]
    }
}

impl ToList<ChannelId> for String {
    fn to_list(&self) -> Vec<ChannelId> {
        let channel = ChannelId::from_str(self);
        if channel.is_ok() {
            return vec![channel.unwrap()];
        }
        Vec::new()
    }
}

impl ToList<ChannelId> for &String {
    fn to_list(&self) -> Vec<ChannelId> {
        let channel = ChannelId::from_str(self);
        if channel.is_ok() {
            return vec![channel.unwrap()];
        }
        Vec::new()
    }
}


pub trait ToMessage {
    fn to_message(&self) -> CreateMessage;
}
impl ToMessage for &str {
    fn to_message(&self) -> CreateMessage {
        CreateMessage::default().content(self.to_string())
    }
}
impl ToMessage for CreateEmbed {
    fn to_message(&self) -> CreateMessage {
        CreateMessage::default().embed(self.clone())
    }
}
impl ToMessage for CreateMessage {
    fn to_message(&self) -> CreateMessage {
        self.clone()
    }
}
impl ToMessage for String {
    fn to_message(&self) -> CreateMessage {
        CreateMessage::default().content(self.to_string())
    }
}
impl ToMessage for &String {
    fn to_message(&self) -> CreateMessage {
        CreateMessage::default().content(self.to_string())
    }
}


pub enum MatchType {
    Exact,
    Fuzzy(String),
    None
}

pub trait Triggerable: Sync {

    fn get_triggers(&self) -> NonEmpty<String>;

    fn is_triggered_by(&self, compare: &String) -> MatchType {

        let compare = compare.to_lowercase();
        let triggers = &self.get_triggers().into_iter()
            .map(|trigger| trigger.to_lowercase())
            .collect::<Vec<_>>();

        // check for exact match
        if triggers.contains(&compare) {
            return MatchType::Exact;
        }

        // check for fuzzy
        for trigger in triggers.into_iter() {
            let threshold = trigger.len() / 3;
            if string_distance(&trigger, &compare) <= threshold
                || trigger.contains(&compare) {
                return MatchType::Fuzzy(trigger.to_string());
            }
        }

        // no match
        MatchType::None
    }

}

pub async fn match_triggerables<'a>
(
    message: &MessageManager,
    compare: &String,
    triggerables: Vec<&dyn Triggerable>
) -> Result<usize> {

    // initialize search
    let mut fuzzy_matches = Vec::new();

    for (i, triggerable) in triggerables.clone().into_iter().enumerate() {
        match triggerable.is_triggered_by(compare) {
            MatchType::Exact => {
                return Ok(i);
            },
            MatchType::Fuzzy(closest_match) => fuzzy_matches.push((triggerable, closest_match)),
            MatchType::None => continue,
        };
    }

    // create buttons
    let buttons = fuzzy_matches.iter().enumerate()
        .map(|(i, (_, closest_match))| {
            CreateButton::new(i.to_string())
                .label(closest_match)
                .style(ButtonStyle::Secondary)
        }).collect::<Vec<_>>();

    if buttons.is_empty() {
        message.reply_failure("No match found").await;
        return Err("No match found".to_string());
    }

    // create embed
    let message_content = message.words.join(" ");
    let hit = message_content.to_lowercase().find(compare).unwrap();
    let highlight = " ".repeat(hit) + &"^".repeat(compare.len());
    let embed = MessageManager::create_embed(|embed| {
        embed
            .title("Did you mean ...")
            .description(&format!("`{message_content}`\n`{}`", highlight))
    }).await;

    // create interaction
    let interaction_helper = message.get_interaction_helper();
    let pressed = interaction_helper.create_buttons(
        embed,
        buttons
    ).await;

    // execute callback
    if let Some(pressed) = pressed {
        let button_index = pressed.parse::<usize>().unwrap();
        let fuzzy_match = fuzzy_matches[button_index].0.get_triggers()[0].clone();
        let triggerables_position = triggerables.into_iter()
            .position(|triggerable| fuzzy_match == triggerable.get_triggers()[0]);
        return Ok(triggerables_position.unwrap());
    }

    Err("No match found".to_string())
}


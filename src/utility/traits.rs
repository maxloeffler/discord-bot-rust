

use serenity::model::prelude::*;
use serenity::builder::{CreateEmbed, CreateMessage};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use std::sync::Arc;
use std::str::FromStr;

#[cfg(feature = "auto_moderation")]
use crate::utility::auto_moder::AutoModerator;
#[cfg(feature = "tickets")]
use crate::utility::ticket_handler::TicketHandler;
use crate::databases::*;


pub trait Singleton: Sized {
    fn get_instance() -> &'static Mutex<Self>;
    fn new() -> Self;
}

macro_rules! impl_singleton {
    ($t:ty) => {
        impl Singleton for $t {
            fn get_instance() -> &'static Mutex<Self> {
                static INSTANCE: Lazy<Arc<Mutex<$t>>> = Lazy::new(|| Arc::new(Mutex::new(<$t>::new())));
                &INSTANCE
            }

            fn new() -> Self {
                <$t>::new()
            }
        }
    };
}

#[cfg(feature = "auto_moderation")]
impl_singleton!(AutoModerator);
#[cfg(feature = "tickets")]
impl_singleton!(TicketHandler);

impl_singleton!(ConfigDB);
impl_singleton!(WarningsDB);
impl_singleton!(MutesDB);
impl_singleton!(FlagsDB);
impl_singleton!(BansDB);
impl_singleton!(AfkDB);


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


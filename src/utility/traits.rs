

use serenity::model::prelude::*;
use serenity::builder::{CreateEmbed, CreateMessage};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use std::sync::Arc;
use std::str::FromStr;

use crate::utility::database::Database;


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

impl_singleton!(Database);


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


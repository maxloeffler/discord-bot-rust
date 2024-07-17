
use regex::Regex;

use std::pin::Pin;
use std::future::Future;


pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Result<T> = std::result::Result<T, String>;

pub fn string_distance(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).filter(|(a, b)| a != b).count()
}

pub struct RegexManager {}

impl RegexManager {

    pub fn get_id_regex() -> Regex {
        Regex::new(r"\d{18,19}").unwrap()
    }

    pub fn get_ping_regex() -> Regex {
        Regex::new(r"<@!?(\d{18,19})>").unwrap()
    }

    pub fn get_channel_regex() -> Regex {
        Regex::new(r"<#(\d{18,19})>").unwrap()
    }

    pub fn get_role_regex() -> Regex {
        Regex::new(r"<@&(\d{18,19})>").unwrap()
    }

    pub fn get_url_regex() -> Regex {
        Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap()
    }
}



use regex::Regex;

use std::pin::Pin;
use std::future::Future;


pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Result<T> = std::result::Result<T, String>;

pub fn string_distance(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).filter(|(a, b)| a != b).count()
}

pub struct RegexManager {
    id: Regex,
    ping: Regex,
    channel: Regex,
    role: Regex,
    url: Regex,
}

impl RegexManager {

    pub fn new() -> RegexManager {
        RegexManager {
            id: Regex::new(r"\d{18,19}").unwrap(),
            ping: Regex::new(r"<@!?(\d{18,19})>").unwrap(),
            channel: Regex::new(r"<#(\d{18,19})>").unwrap(),
            role: Regex::new(r"<@&(\d{18,19})>").unwrap(),
            url: Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap(),
        }
    }

    pub fn get_id_regex(&self) -> Regex {
        self.id.clone()
    }

    pub fn get_ping_regex(&self) -> Regex {
        self.ping.clone()
    }

    pub fn get_channel_regex(&self) -> Regex {
        self.channel.clone()
    }

    pub fn get_role_regex(&self) -> Regex {
        self.role.clone()
    }

    pub fn get_url_regex(&self) -> Regex {
        self.url.clone()
    }

}


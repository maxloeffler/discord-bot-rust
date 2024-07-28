
use regex::Regex;

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;


pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Result<T> = std::result::Result<T, String>;

// Implement Levenshtein distance
// https://www.wikiwand.com/en/Levenshtein_distance
pub fn string_distance(a: &str, b: &str) -> usize {

    if a.is_empty() {
        return b.len();
    }

    if b.is_empty() {
        return a.len();
    }

    if a.chars().next().unwrap() == b.chars().next().unwrap() {
        return string_distance(&a[1..], &b[1..]);
    }

    1 + [
        string_distance(&a[1..], b),
        string_distance(a, &b[1..]),
        string_distance(&a[1..], &b[1..]),
    ].iter().min().unwrap().clone()

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



use cached::proc_macro::cached;
use cached::SizedCache;

use regex::{Regex, escape};

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;


pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type Result<T> = std::result::Result<T, String>;

// Implement Levenshtein distance
// https://www.wikiwand.com/en/Levenshtein_distance
#[cached(
    ty = "SizedCache<(String, String), usize>",
    create = "{ SizedCache::with_size(200) }",
    convert = r#"{ (a.to_string(), b.to_string()) }"#
)]
pub fn string_distance(a: &str, b: &str) -> usize {

    if a.is_empty() {
        return b.len();
    }

    if b.is_empty() {
        return a.len();
    }

    if a[0..1] == b[0..1] {
        return string_distance(&a[1..], &b[1..]);
    }

    1 + [
        string_distance(&a[1..], b),
        string_distance(a, &b[1..]),
        string_distance(&a[1..], &b[1..]),
    ].iter().min().unwrap().clone()

}

pub fn parse_time(input: impl Into<String>) -> Result<u64> {

    let re = Regex::new(r"(?i)(?P<value>\d+)(?P<unit>[dhms])").map_err(|e| e.to_string())?;
    let mut total_seconds = 0;

    let input = &input.into();
    let captures = re.captures_iter(input).collect::<Vec<_>>();
    if captures.is_empty() {
        return Err("Invalid time format".to_string());
    }

    for cap in captures.iter() {
        let value: u64 = cap["value"].parse().map_err(|_| "Invalid number".to_string())?;
        let unit = &cap["unit"];

        // Convert the parsed value to seconds based on the unit
        let seconds = match unit.to_lowercase().as_str() {
            "w" => value * 604_800, // weeks
            "d" => value * 86_400,  // days
            "h" => value * 3_600,   // hours
            "m" => value * 60,      // minutes
            "s" => value,           // seconds
            _ => return Err(format!("Invalid unit: {}", unit)),
        };

        total_seconds += seconds;
    }

    Ok(total_seconds)
}

pub fn binary_search<T, O: Into<i64>>(array: &Vec<T>, target: O, eval: fn(&T) -> O) -> usize {

    let mut left = 0;
    let mut right = array.len();
    let target = target.into();

    while left < right {
        let middle = (left + right) / 2;
        match eval(&array[middle]).into() {
            x if x == target => return middle,
            x if x < target  => left = middle + 1,
            _                => right = middle,
        }
    }

    left
}

pub struct RegexManager {}

impl RegexManager {

    pub fn get_id_regex() -> Regex {
        Regex::new(r"\d{18,19}").unwrap()
    }

    // maybe used in the future
    #[allow(unused)]
    pub fn get_ping_regex() -> Regex {
        Regex::new(r"<@!?(\d{18,19})>").unwrap()
    }

    // maybe used in the future
    #[allow(unused)]
    pub fn get_channel_regex() -> Regex {
        Regex::new(r"<#(\d{18,19})>").unwrap()
    }

    pub fn get_role_regex() -> Regex {
        Regex::new(r"<@&(\d{18,19})>").unwrap()
    }

    pub fn get_url_regex() -> Regex {
        Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap()
    }

    pub fn escape(text: &str) -> String {
        if Regex::new(text).is_ok() {
            return text.to_string();
        }
        escape(text)
    }
}



use regex::Regex;

use crate::utility::message_manager::MessageManager;


pub enum FilterType {
    Slur,
    Link,
    Fine
}

pub struct ChatFilter {
    pub filter: FilterType,
    pub context: String
}

pub struct ChatFilterManager {
    message: MessageManager,
    slurs: Vec<String>,
    domain_regex: Regex,
    domain_whitelist: Vec<String>,
}

impl ChatFilterManager {

    pub fn new(message: MessageManager) -> ChatFilterManager {
        let slurs = vec![
            "nigga",
            "nigger",
            "niglet",
            "faggot",
            "fag",
            "retard",
            "chink",
            "dyke",
            "lesbo",
            "gypsy",
            "gypped",
            "ching chong",
            "tranny",
            "beaner"
        ].into_iter().map(|str| str.to_string()).collect::<Vec<String>>();
        let domain_whitelist = vec![
            "tenor.com",
            "giphy.com",
            "discord.com",
            "spotify.com",
            "spotify.link"
        ].into_iter().map(|str| str.to_string()).collect::<Vec<String>>();
        ChatFilterManager {
            message,
            slurs,
            domain_regex: Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap(),
            domain_whitelist
        }
    }

    pub async fn filter(&self) -> ChatFilter {

        let words = self.message.payload(None, None)
            .split_whitespace()
            .map(|str| str.to_string())
            .collect::<Vec<String>>();

        for word in words {

            if self.slurs.contains(&word) {
                return ChatFilter {
                    filter: FilterType::Slur,
                    context: word.to_string()
                };
            }

            if self.domain_regex.is_match(word.as_str()) {
                let mut external = true;
                for domain in self.domain_whitelist.clone() {
                    if word.contains(&domain) {
                        external = false;
                        break;
                    }
                }
                if external {
                    return ChatFilter {
                        filter: FilterType::Link,
                        context: word.to_string()
                    };
                }
            }
        }

        ChatFilter {
            filter: FilterType::Fine,
            context: String::new()
        }
    }

}

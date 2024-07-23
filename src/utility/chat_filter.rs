
use crate::utility::*;


#[derive(PartialEq)]
pub enum FilterType {
    Slur,
    Link,
    Fine
}

pub struct ChatFilter {
    pub filter: FilterType,
    pub context: String
}

pub struct ChatFilterManager<'a> {
    message: &'a MessageManager,
    slurs: Vec<String>,
    domain_whitelist: Vec<String>,
}

impl<'a> ChatFilterManager<'_> {

    pub fn new(message: &'a MessageManager) -> ChatFilterManager<'a> {
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

            let url_regex = RegexManager::get_url_regex();
            if url_regex.is_match(word.as_str()) {
                let mut external = true;
                for domain in &self.domain_whitelist {
                    if word.contains(domain) {
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

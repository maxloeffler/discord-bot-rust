
use serenity::all::ChannelId;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use std::sync::Arc;

use crate::utility::*;
use crate::databases::*;
use crate::impl_singleton;


#[derive(PartialEq)]
pub enum FilterType {
    Slur,
    Link,
    Fine
}

impl FilterType {
    pub fn to_string(&self) -> String {
        match self {
            FilterType::Slur => "slur",
            FilterType::Link => "link",
            FilterType::Fine => "fine"
        }.to_string()
    }
}

pub struct Filter {
    pub filter_type: FilterType,
    pub context: String
}

pub struct ChatFilter {
    slurs: Vec<String>,
    domains: Vec<String>,
    music_domains: Vec<String>,
}

impl ChatFilter {

    pub fn new() -> ChatFilter {
        ChatFilter {
            slurs: vec![
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
            ].into_iter().map(|slur| slur.to_string()).collect(),
            domains: vec![
                "tenor.com",
                "giphy.com",
                "discord.com",
                "spotify.com",
                "spotify.link"
            ].into_iter().map(|domain| domain.to_string()).collect(),
            music_domains: vec![
                "youtube.com",
                "soundcloud.com"
            ].into_iter().map(|domain| domain.to_string()).collect()
        }
    }

    pub async fn apply(&self, message: &MessageManager) -> Filter {

        // fetch channel
        let channel = message.resolve_guild_channel().await;

        // do not moderate dms
        if channel.is_none() {
            return Filter {
                filter_type: FilterType::Fine,
                context: message.payload(None, None)
            };
        }
        let channel = channel.unwrap();

        // no filtering in ticket channels
        #[cfg(feature = "tickets")]
        if TicketHandler::get_instance().get_ticket(&channel.id).await.is_some() {
            return Filter {
                filter_type: FilterType::Fine,
                context: message.payload(None, None)
            };
        }

        // fetch additional roles and channels
        let category_music: ChannelId = ConfigDB::get_instance()
            .get("category_music").await.unwrap().into();
        let link_perm_roles = message.get_resolver().resolve_role(vec!["Level 30+", "Booster"]).await.unwrap();
        let has_link_perms = message.has_role(link_perm_roles).await;

        // perform word based analysis
        for word in message.words.iter() {
            let word_lc = word.to_lowercase();

            // check for slurs
            for slur in &self.slurs {
                if word_lc.contains(slur) {
                    return Filter {
                        filter_type: FilterType::Slur,
                        context: word.to_string()
                    };
                }
            }

            // allows permitted users to post links
            if has_link_perms {
                continue;
            }

            // check for links
            let url_regex = RegexManager::get_url_regex();
            if url_regex.is_match(word.as_str()) {
                let mut external = true;

                // compare against regular list of whitelisted domains
                for domain in &self.domains {
                    if word.contains(domain) {
                        external = false;
                        break;
                    }
                }

                // check music category
                if external {
                    if let Some(category) = channel.parent_id {
                        if category == category_music {
                            for music_domain in &self.music_domains {
                                if word_lc.contains(music_domain) {
                                    external = false;
                                    break;
                                }
                            }
                        }
                    }
                }

                // if non-whitelisted domain is hit
                if external {
                    return Filter {
                        filter_type: FilterType::Link,
                        context: word.to_string()
                    };
                }
            }
        }

        Filter {
            filter_type: FilterType::Fine,
            context: String::new()
        }
    }

}

impl_singleton!(ChatFilter);

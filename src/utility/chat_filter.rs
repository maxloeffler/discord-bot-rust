
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
                "beaner",
                "batty boy",
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

        // perform content analysis
        let content = message.payload(None, None).to_lowercase();

        // check for slurs
        for slur in &self.slurs {
            if let Some(index) = content.find(slur) {

                // get a ±7 character context window
                let lower_bound = index.saturating_sub(             7).clamp(0, content.len());
                let upper_bound = index.saturating_add(slur.len() + 7).clamp(0, content.len());

                let context_window = &content[lower_bound..upper_bound].trim();
                let prefix = match lower_bound { 0 => "", _ => "[…] " };
                let suffix = match upper_bound { len if len == content.len() => "", _ => " […]" };

                return Filter {
                    filter_type: FilterType::Slur,
                    context: format!("{}{}{}", prefix, context_window, suffix)
                };
            }
        }

        // fetch additional roles and channels
        let category_music: ChannelId = ConfigDB::get_instance()
            .get("category_music").await.unwrap().into();
        let link_perm_roles = message.resolve_role(vec!["Level 30+", "Booster"]).await;

        // sometimes the role cache of a guild is randomly empty
        // in this case, we allow all users to post links
        let has_link_perms = link_perm_roles.is_none()
            || message.has_role(link_perm_roles.unwrap()).await;

        if !has_link_perms {

            let url_regex = RegexManager::get_url_regex();

            // check for links (first perform a low-cost check)
            if url_regex.is_match(&content) {

                let link = url_regex.find(&content).unwrap().as_str();
                if !link.ends_with(".gif") {

                    let mut allowed_link = false;

                    // compare against regular list of whitelisted domains
                    for whitelisted_domain in &self.domains {
                        if link.contains(whitelisted_domain) {
                            allowed_link = true;
                            break;
                        }
                    }

                    // check music category
                    if !allowed_link {
                        if let Some(category) = channel.parent_id {
                            if category == category_music {
                                for whitelisted_music_domain in &self.music_domains {
                                    if link.contains(whitelisted_music_domain) {
                                        allowed_link = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // if domain is not allowed
                    if !allowed_link {
                        return Filter {
                            filter_type: FilterType::Link,
                            context: link.to_string()
                        };
                    }
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

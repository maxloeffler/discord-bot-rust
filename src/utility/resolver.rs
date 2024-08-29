
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::all::{CacheHttp, Cache, Http};
use cached::proc_macro::cached;
use cached::SizedCache;

use std::sync::Arc;

use crate::utility::*;


#[derive(Debug, Clone)]
pub struct Resolver {
    ctx: Context,
    guild_id: Option<GuildId>
}

impl CacheHttp for Resolver {
    fn http(&self) -> &Http {
        self.http()
    }
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(self.cache())
    }
}

impl AsRef<Http> for Resolver {
    fn as_ref(&self) -> &Http {
        self.http()
    }
}

impl Resolver {

    pub fn new(ctx: Context, guild_id: Option<GuildId>) -> Self {
        Resolver { ctx: ctx, guild_id: guild_id }
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    pub fn http(&self) -> &Arc<Http> {
        &self.ctx.http
    }

    pub fn cache(&self) -> &Arc<Cache> {
        &self.ctx.cache
    }

    pub async fn resolve_guild(&self, guild_id: Option<GuildId>) -> Option<Guild> {

        let guild_id = match guild_id {
            None => self.guild_id,
            guild @ _ => guild
        };

        if let Some(guild_id) = guild_id {

            // attempt to get guild from cache
            if let Some(guild) = guild_id.to_guild_cached(&self.ctx.cache) {
                return Some(guild.clone());
            }

        }
        None
    }

    pub async fn resolve_user(&self, user_id: UserId) -> Option<User> {

        // first, attempt to get user from cache
        if let Some(user) = user_id.to_user_cached(&self.ctx.cache) {
            return Some(user.clone());
        }

        // if cache fails, attempt to get user over discord api
        if let Ok(user) = self.ctx.http.get_user(user_id).await {
            return Some(user);
        }

        None
    }

    pub async fn resolve_member(&self, user: &User) -> Option<Member> {

        if let Some(guild_id) = self.guild_id {

            // first, attempt to get member from cache
            if let Some(guild) = self.ctx.cache.guild(guild_id) {
                let member = guild.members.get(&user.id);
                if let Some(member) = member {
                    return Some(member.clone());
                }
            }

            // if cache fails, attempt to get member over discord api
            if let Ok(member) = guild_id.member(&self.ctx.http, user.id).await {
                return Some(member);
            }

        }
        None
    }

    pub async fn resolve_role(&self, role_name: impl ToList<&str>) -> Option<Vec<Role>> {

        if let Some(guild_id) = self.guild_id {

            // first, attempt to get roles from cache
            if let Some(guild) = self.ctx.cache.guild(guild_id) {
                let values: Vec<_> = role_name.to_list()
                    .into_iter()
                    .flat_map(|name| {
                        guild.roles.values().find(|role| role.name == name)
                    })
                    .cloned()
                    .collect();
                if values.len() == role_name.to_list().len() {
                    return Some(values);
                }
            }

            // if cache fails, attempt to get roles over discord api
            if let Ok(guild_roles) = guild_id.roles(&self.ctx.http).await {
                let values: Vec<_> = role_name.to_list()
                    .into_iter()
                    .flat_map(|name| {
                        guild_roles.values().find(|role| role.name == name)
                    })
                    .cloned()
                    .collect();
                if values.len() == role_name.to_list().len() {
                    return Some(values);
                }
            }

        }
        None
    }

    pub async fn guild_channels(&self) -> Option<Vec<GuildChannel>> {

        if let Some(guild_id) = self.guild_id {

            // first, attempt to get channels from cache
            if let Some(guild) = self.ctx.cache.guild(guild_id) {
                let channels = guild.channels
                    .values()
                    .cloned()
                    .collect();
                return Some(channels);
            }

            // if cache fails, attempt to get channels over discord api
            let channels = guild_id.channels(&self.http()).await;
            if let Ok(channels) = channels {
                let values = channels
                    .values()
                    .cloned()
                    .collect();
                return Some(values);
            }

        }
        None
    }

    pub async fn resolve_guild_channel(&self, channel_id: ChannelId) -> Option<GuildChannel> {
        let channels = self.guild_channels().await;
        if let Some(channels) = channels {
            let channel = channels
                .into_iter()
                .find(|channel| channel.id == channel_id);
            return channel;
        }
        None
    }

    pub async fn resolve_category_channels(&self, category_id: ChannelId) -> Option<Vec<GuildChannel>> {
        let channels = self.guild_channels().await;
        if let Some(channels) = channels {
            let category_channels = channels
                .into_iter()
                .filter(|channel| channel.parent_id == Some(category_id))
                .collect();
            return Some(category_channels);
        }
        None
    }

    pub async fn resolve_message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        let message = self.http().get_message(channel_id, message_id).await;
        match message {
            Ok(message) => Some(message),
            Err(_) => match self.ctx.cache.message(channel_id, message_id) {
                Some(message) => Some(message.clone()),
                None => None
            }
        }
    }

    pub fn resolve_name(&self, user: &User) -> String {
        user.global_name.clone().unwrap_or(user.name.clone())
    }

    pub async fn has_role(&self, user: &User, roles: impl ToList<RoleId>) -> bool {
        let member = self.resolve_member(user).await;
        if let Some(member) = member {
            return roles.to_list().iter().any(|role| member.roles.contains(role));
        }
        false
    }

    pub async fn is_admin(&self, user: &User) -> bool {
        let role_id = self.resolve_role("Administrator")
            .await.unwrap()[0].id;
        self.has_role(user, role_id).await
    }

    pub async fn is_headmod(&self, user: &User) -> bool {
        let role_ids: Vec<_> = self.resolve_role(vec!["Administrator", "Head Moderator"])
            .await.unwrap().iter().map(|role| role.id).collect();
        self.has_role(user, role_ids).await
    }

    pub async fn is_mod(&self, user: &User) -> bool {
        let role_ids: Vec<_> = self.resolve_role(vec!["Administrator", "Head Moderator", "Moderator"])
            .await.unwrap().iter().map(|role| role.id).collect();
        self.has_role(user, role_ids).await
    }

    pub async fn is_trial(&self, user: &User) -> bool {
        let role_ids: Vec<_> = self.resolve_role(vec!["Administrator", "Head Moderator", "Moderator", "Trial Moderator"])
            .await.unwrap().iter().map(|role| role.id).collect();
        self.has_role(user, role_ids).await
    }

}


#[cached(
    ty = "SizedCache<UserId, bool>",
    create = "{ SizedCache::with_size(100) }",
    convert = r#"{ user.id }"#
)]
pub async fn is_trial(resolver: &Resolver, user: &User) -> bool {
    resolver.is_trial(user).await
}


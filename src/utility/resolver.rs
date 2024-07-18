
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::http::Http;
use serenity::cache::Cache;

use std::sync::Arc;

use crate::utility::*;


#[derive(Clone)]
pub struct Resolver {
    ctx: Context,
    guild_id: Option<GuildId>
}

impl Resolver {

    pub fn new(ctx: Context, guild_id: Option<GuildId>) -> Resolver {
        Resolver { ctx: ctx, guild_id: guild_id }
    }

    pub fn ctx(&self) -> Context {
        self.ctx.clone()
    }

    pub fn http(&self) -> Arc<Http> {
        self.ctx.clone().http
    }

    pub fn cache(&self) -> Arc<Cache> {
        self.ctx.clone().cache
    }

    pub async fn resolve_user(&self, user_id: UserId) -> Option<User> {
        let user = self.ctx.http.get_user(user_id).await;
        match user {
            Ok(user) => Some(user),
            Err(_) => None
        }
    }

    pub async fn resolve_member(&self, user: User) -> Option<Member> {
        if let Some(guild) = self.guild_id {
            let member = guild.member(&self.ctx.http, user.id).await;
            return match member {
                Ok(member) => Some(member),
                Err(_) => None
            };
        }
        None
    }

    pub async fn resolve_role(&self, role_name: impl ToList<&str>) -> Option<Vec<Role>> {
        if let Some(guild_id) = self.guild_id {
            let guild_roles = guild_id.roles(&self.ctx.http).await.unwrap();
            let mut values = Vec::new();
            for name in role_name.to_list() {
                for role in guild_roles.values() {
                    if role.name == name {
                        values.push(role.clone());
                    }
                }
            }
            if values.len() == role_name.to_list().len() {
                return Some(values);
            }
        }
        None
    }

    pub async fn resolve_channel(&self, channel_name: impl ToList<ChannelId>) -> Option<ChannelId> {
        if let Some(guild_id) = self.guild_id {
            let guild_channels = guild_id.channels(&self.ctx.http).await.unwrap();
            for channel in channel_name.to_list() {
                for guild_channel in guild_channels.values() {
                    if guild_channel.id == channel {
                        return Some(channel);
                    }
                }
            }
        }
        None
    }

    pub async fn resolve_message(&self, channel_id: ChannelId, message_id: MessageId) -> Option<Message> {
        let message = self.ctx.http.get_message(channel_id, message_id).await;
        match message {
            Ok(message) => Some(message),
            Err(_) => match self.ctx.cache.message(channel_id, message_id) {
                Some(message) => Some(message.clone()),
                None => None
            }
        }
    }

    pub fn resolve_name(&self, user: User) -> String {
        user.clone().global_name.unwrap_or(user.clone().name)
    }

    pub async fn has_role(&self, user: User, roles: impl ToList<RoleId>) -> bool {
        if let Some(guild) = self.guild_id {
            for role in roles.to_list() {
                let has_role = user.has_role(self.ctx.clone(), guild.clone(), role).await;
                if let Ok(true) = has_role {
                    return true;
                }
            }
        }
        false
    }

    pub async fn get_roles(&self, user: User) -> Option<Vec<RoleId>> {
        let member = self.resolve_member(user).await;
        if member.is_some() {
            return Some(member.unwrap().roles);
        }
        None
    }

    pub async fn is_admin(&self, user: User) -> bool {
        let role_id = self.resolve_role("Administrator")
            .await.unwrap()[0].id;
        self.has_role(user, role_id).await
    }

    pub async fn is_headmod(&self, user: User) -> bool {
        let role_ids: Vec<_> = self.resolve_role(vec!["Administrator", "Head Moderator"])
            .await.unwrap().iter().map(|role| role.id).collect();
        self.has_role(user, role_ids).await
    }

    pub async fn is_mod(&self, user: User) -> bool {
        let role_ids: Vec<_> = self.resolve_role(vec!["Administrator", "Head Moderator", "Moderator"])
            .await.unwrap().iter().map(|role| role.id).collect();
        self.has_role(user, role_ids).await
    }

    pub async fn is_trial(&self, user: User) -> bool {
        let role_ids: Vec<_> = self.resolve_role(vec!["Administrator", "Head Moderator", "Moderator", "Trial Moderator"])
            .await.unwrap().iter().map(|role| role.id).collect();
        self.has_role(user, role_ids).await
    }

}

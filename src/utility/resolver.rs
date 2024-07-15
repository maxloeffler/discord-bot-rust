
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::utility::traits::{Singleton, ToList};
use crate::databases::*;


pub struct Resolver {}

impl Resolver {

    pub fn new() -> Resolver {
        Resolver {}
    }

    pub async fn get_user(&self, ctx: Context, user_id: UserId) -> Option<User> {
        let user = ctx.http.get_user(user_id).await;
        match user {
            Ok(user) => Some(user),
            Err(_) => None
        }
    }

    pub async fn get_member(&self, ctx: Context, guild_id: Option<GuildId>, user: User) -> Option<Member> {
        if let Some(guild) = guild_id {
            let member = guild.member(&ctx.http, user.id).await;
            return match member {
                Ok(member) => Some(member),
                Err(_) => None
            };
        }
        None
    }

    pub async fn has_role(&self, ctx: Context, guild_id: Option<GuildId>, user: User, roles: impl ToList<RoleId>) -> bool {
        if let Some(guild) = guild_id {
            for role in roles.to_list() {
                let has_role = user.has_role(ctx.clone(), guild.clone(), role).await;
                if let Ok(true) = has_role {
                    return true;
                }
            }
        }
        false
    }

    pub async fn get_roles(&self, ctx: Context, guild_id: Option<GuildId>, user: User) -> Option<Vec<RoleId>> {
        let member = self.get_member(ctx, guild_id, user).await;
        if member.is_some() {
            return Some(member.unwrap().roles);
        }
        None
    }

    pub async fn is_admin(&self, ctx: Context, guild_id: Option<GuildId>, user: User) -> bool {
        let role_id = ConfigDB::get_instance().lock().await
            .get("role_admin").await.unwrap();
        self.has_role(ctx, guild_id, user, role_id).await
    }

    pub async fn is_headmod(&self, ctx: Context, guild_id: Option<GuildId>, user: User) -> bool {
        let role_ids = ConfigDB::get_instance().lock().await
            .get_multiple(vec!["role_admin", "role_headmod"]).await.unwrap();
        self.has_role(ctx, guild_id, user, role_ids).await
    }

    pub async fn is_mod(&self, ctx: Context, guild_id: Option<GuildId>, user: User) -> bool {
        let role_ids = ConfigDB::get_instance().lock().await
            .get_multiple(vec!["role_admin", "role_headmod", "role_mod"]).await.unwrap();
        self.has_role(ctx, guild_id, user, role_ids).await
    }

    pub async fn is_trial(&self, ctx: Context, guild_id: Option<GuildId>, user: User) -> bool {
        let role_ids = ConfigDB::get_instance().lock().await
            .get_multiple(vec!["role_admin", "role_headmod", "role_mod", "role_trial"]).await.unwrap();
        self.has_role(ctx, guild_id, user, role_ids).await
    }

}

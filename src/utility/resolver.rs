
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::utility::traits::{Singleton, ToList};
use crate::utility::database::{Database, DB};


pub struct Resolver {}

impl Resolver {

    pub fn new() -> Resolver {
        Resolver {}
    }

    pub async fn get_guild(&self, ctx: Context) -> Option<Guild> {
        None
    }

    pub async fn get_user(&self, ctx: Context, user_id: UserId) -> Option<User> {
        let user = ctx.http.get_user(user_id).await;
        match user {
            Ok(user) => Some(user),
            Err(_) => None
        }
    }

    pub async fn get_member(&self, ctx: Context, user: User) -> Option<Member> {
        let guild = self.get_guild(ctx.clone()).await;
        if let Some(guild) = guild {
            let member = guild.member(&ctx.http, user.id).await;
            return match member {
                Ok(member) => Some(member.into_owned()),
                Err(_) => None
            };
        }
        None
    }

    pub async fn has_role(&self, ctx: Context, user: User, roles: impl ToList<RoleId>) -> bool {
        let guild = self.get_guild(ctx.clone()).await;
        if let Some(guild) = guild {
            for role in roles.to_list() {
                let has_role = user.has_role(ctx.clone(), guild.clone(), role).await;
                if let Ok(true) = has_role {
                    return true;
                }
            }
        }
        false
    }

    pub async fn get_roles(&self, ctx: Context, user: User) -> Option<Vec<RoleId>> {
        let member = self.get_member(ctx, user).await;
        if member.is_some() {
            return Some(member.unwrap().roles);
        }
        None
    }

    pub async fn is_admin(&self, ctx: Context, user: User) -> bool {
        let role_id = Database::get_instance().lock().await
            .get(DB::Config, "role_admin_id").await;
        match role_id {
            Some(role) => self.has_role(ctx, user, role).await,
            _ => false
        }
    }

    pub async fn is_headmod(&self, ctx: Context, user: User) -> bool {
        let role_ids = Database::get_instance().lock().await
            .get_multiple(DB::Config, vec!["role_admin_id",
                                           "role_headmod_id"]).await;
        match role_ids {
            Some(roles) => self.has_role(ctx, user, roles).await,
            _ => false
        }
    }

    pub async fn is_mod(&self, ctx: Context, user: User) -> bool {
        let role_ids = Database::get_instance().lock().await
            .get_multiple(DB::Config, vec!["role_admin_id",
                                           "role_headmod_id",
                                           "role_mod_id"]).await;
        match role_ids {
            Some(roles) => self.has_role(ctx, user, roles).await,
            _ => false
        }
    }

    pub async fn is_trial(&self, ctx: Context, user: User) -> bool {
        let role_ids = Database::get_instance().lock().await
            .get_multiple(DB::Config, vec!["role_admin_id",
                                           "role_headmod_id",
                                           "role_mod_id",
                                           "role_trial_id"]).await;
        match role_ids {
            Some(roles) => self.has_role(ctx, user, roles).await,
            _ => false
        }
    }

}

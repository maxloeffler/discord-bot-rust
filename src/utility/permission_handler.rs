
use serenity::model::permissions::Permissions;
use serenity::model::channel::{PermissionOverwriteType, PermissionOverwrite};
use serenity::model::channel::GuildChannel;
use serenity::model::id::{RoleId, UserId};
use futures::StreamExt;

use crate::utility::*;


pub struct PermissionHandler<'a> {
    resolver: &'a Resolver,
    channel: &'a GuildChannel
}

impl<'a> PermissionHandler<'_> {

    pub fn new(resolver: &'a Resolver, channel: &'a GuildChannel) -> PermissionHandler<'a> {
        PermissionHandler {
            resolver: resolver,
            channel: channel,
        }
    }

    async fn update_permissions(&self, overwrites: Vec<PermissionOverwrite>) {
        for overwrite in overwrites {
            let _ = self.channel.create_permission(self.resolver, overwrite).await;
        }
    }

    pub async fn allow_role(&self, allows: impl ToList<Permissions>, roles: &impl ToList<RoleId>) {
        self.role(allows, Permissions::empty(), roles).await;
    }

    pub async fn deny_role(&self, denies: impl ToList<Permissions>, roles: &impl ToList<RoleId>) {
        self.role(Permissions::empty(), denies, roles).await;
    }

    pub async fn role(&self, allows: impl ToList<Permissions>, denies: impl ToList<Permissions>, roles: &impl ToList<RoleId>) {

        // create singluar allow permission
        let mut allow = Permissions::empty();
        allows.to_list()
            .into_iter()
            .for_each(|perm| {
                allow = allow.union(perm);
            });

        // create singular deny permission
        let mut deny = Permissions::empty();
        denies.to_list()
            .into_iter()
            .for_each(|perm| {
                deny = deny.union(perm);
            });

        // create overwrites
        let overwrites: Vec<_> = roles.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: allow,
                deny:  deny,
                kind: PermissionOverwriteType::Role(id.into())
            }
        }).collect();

        // set permissions
        self.update_permissions(overwrites).await;
    }

    pub async fn allow_member(&self, allows: impl ToList<Permissions>, members: &impl ToList<UserId>) {
        self.member(allows, Permissions::empty(), members).await;
    }

    pub async fn deny_member(&self, denies: impl ToList<Permissions>, members: &impl ToList<UserId>) {
        self.member(Permissions::empty(), denies, members).await;
    }

    pub async fn member(&self, allows: impl ToList<Permissions>, denies: impl ToList<Permissions>, members: &impl ToList<UserId>) {

        // create singluar allow permission
        let mut allow = Permissions::empty();
        allows.to_list()
            .into_iter()
            .for_each(|perm| {
                allow = allow.union(perm);
            });

        // create singular deny permission
        let mut deny = Permissions::empty();
        denies.to_list()
            .into_iter()
            .for_each(|perm| {
                deny = deny.union(perm);
            });

        // create overwrites
        let overwrites: Vec<_> = members.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: allow,
                deny:  deny,
                kind: PermissionOverwriteType::Member(id.into())
            }
        }).collect();

        // set permissions
        self.update_permissions(overwrites).await;
    }

    pub async fn remove_overwrites(&self, target: UserId) {
        let permission = PermissionOverwriteType::Member(target);
        let _ = self.channel.delete_permission(self.resolver, permission).await;
    }

}

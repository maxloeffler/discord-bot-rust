
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
        let present_overwrites = &self.channel.permission_overwrites;
        let new_overwrites = overwrites.into_iter()
            .filter(|overwrite| !present_overwrites.contains(&overwrite));
        futures::stream::iter(new_overwrites)
            .for_each_concurrent(None, |overwrite| async move {
                let _ = self.channel.create_permission(self.resolver.ctx(), overwrite).await;
            }).await;
    }

    pub async fn allow_role(&self, permission: &Permissions, ids: impl ToList<RoleId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: *permission,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(id.into())
            }
        }).collect();
        self.update_permissions(overwrites).await;
    }

    pub async fn deny_role(&self, permission: &Permissions, ids: impl ToList<RoleId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: *permission,
                kind: PermissionOverwriteType::Role(id.into())
            }
        }).collect();
        self.update_permissions(overwrites).await;
    }

    pub async fn allow_member(&self, permission: &Permissions, ids: impl ToList<UserId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: *permission,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(id.into())
            }
        }).collect();
        self.update_permissions(overwrites).await;
    }

    pub async fn deny_member(&self, permission: &Permissions, ids: impl ToList<UserId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: *permission,
                kind: PermissionOverwriteType::Member(id.into())
            }
        }).collect();
        self.update_permissions(overwrites).await;
    }

}

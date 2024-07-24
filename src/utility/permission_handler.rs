
use serenity::model::permissions::Permissions;
use serenity::model::channel::{PermissionOverwriteType, PermissionOverwrite};
use serenity::model::channel::GuildChannel;
use serenity::model::id::{RoleId, UserId};

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

    pub async fn allow_role(&self, permission: &Permissions, ids: impl ToList<RoleId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: *permission,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(id.into())
            }
        }).collect();
        for overwrite in overwrites {
            let _ = self.channel.create_permission(self.resolver.ctx(), overwrite).await;
        }
    }

    pub async fn deny_role(&self, permission: &Permissions, ids: impl ToList<RoleId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: *permission,
                kind: PermissionOverwriteType::Role(id.into())
            }
        }).collect();
        for overwrite in overwrites {
            let _ = self.channel.create_permission(self.resolver.ctx(), overwrite).await;
        }
    }

    pub async fn allow_member(&self, permission: &Permissions, ids: impl ToList<UserId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: *permission,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(id.into())
            }
        }).collect();
        for overwrite in overwrites {
            let _ = self.channel.create_permission(self.resolver.ctx(), overwrite).await;
        }
    }

    pub async fn deny_member(&self, permission: &Permissions, ids: impl ToList<UserId>) {
        let overwrites: Vec<_> = ids.to_list().iter().map(|id| {
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: *permission,
                kind: PermissionOverwriteType::Member(id.into())
            }
        }).collect();
        for overwrite in overwrites {
            let _ = self.channel.create_permission(self.resolver.ctx(), overwrite).await;
        }
    }

}

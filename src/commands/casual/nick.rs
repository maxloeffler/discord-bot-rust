
use serenity::builder::EditMember;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;


pub struct NicknameCommand;

impl Command for NicknameCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'a, bool> {
        Box::pin(async move {
            let role = message.resolve_role("Level 10+").await.unwrap();
            message.has_role(role).await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Casual,
            nonempty!["nick".to_string(), "nickname".to_string()]
        )
            .add_required(vec!["user", "nickname"])
            .example("@Poggy Poggor")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();
                let target_is_self = target.id == message.get_author().id;

                if !target_is_self {

                    // only staff can change others nicknames
                    if !message.is_trial().await {
                        message.reply_failure("You cannot change the nicknames of others.").await;
                        return;
                    }

                    // staff changing other staffs nickname
                    if message.get_resolver().is_trial(&target).await {
                        message.reply_failure("You cannot change the nicknames of other staff.").await;
                        return;
                    }

                }

                let nickname = message.payload_without_mentions(None, None);

                // cannot change nickname to empty
                if nickname.is_empty() {
                    message.reply_failure("No nickname given.").await;
                    return;
                }

                // nickname too long
                if nickname.len() > 32 {
                    message.reply_failure("Nickname can be at most 32 characters long.").await;
                    return;
                }

                // change nickname
                let member = message.get_resolver().resolve_member(target).await;
                if let Some(mut member) = member {
                    let edit = EditMember::default().nickname(nickname);
                    member.edit(&message, edit).await.unwrap();
                }

                let _ = message.reply_success().await;
            }
        )
    }
}



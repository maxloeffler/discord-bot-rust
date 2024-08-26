
use serenity::builder::CreateEmbedFooter;
use serenity::model::user::User;
use nonempty::NonEmpty;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashSet;

use crate::utility::*;


pub struct CommandParams {
    pub message: MessageManager,
    pub target: Option<User>,
    pub number: Option<i64>,
}

impl CommandParams {
    pub fn new(message: MessageManager) -> Self {
        Self { message, target: None, number: None }
    }
    pub fn set_target(mut self, target: Option<User>) -> Self {
        self.target = target;
        self
    }
    pub fn set_number(mut self, number: Option<i64>) -> Self {
        self.number = number;
        self
    }
}

pub trait Command: Send + Sync {

    fn permission<'a>(&'a self, _message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move { true })
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()>;

    fn define_usage(&self) -> UsageBuilder;

    fn display_usage(&self, params: CommandParams, title: String) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let message = &params.message;
                let usage = self.define_usage().build(&message.get_prefix().unwrap());
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title(title)
                        .description(&usage)
                        .footer(CreateEmbedFooter::new(
                            format!("Syntax Legend: () = required, [] = optional"),
                        ))
                }).await;
                let _ = message.reply(embed.to_message()).await;
            }
        )
    }

    fn invalid_usage(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move { self.display_usage(params, "Invalid Usage!".to_string()).await }
        )
    }

}

impl Triggerable for Box<dyn Command> {

    fn get_triggers(&self) -> NonEmpty<String> {
        self.define_usage().triggers
    }

}

pub struct UserDecorator {
    pub command: Box<dyn Command>,
}

impl UserDecorator {

    async fn get_target(&self, message: &MessageManager) -> Option<User> {
        let mentions = message.get_mentions().await;
        match mentions.len() {

            0 => {
                // prepare message
                let embed = MessageManager::create_embed(|embed| {
                    embed
                        .title("Select a user!")
                        .description("Here are some suggestions ...")
                }).await;

                // prepare dropdown options
                let last_messages = message.last_messages(20).await;
                let mut users_set = HashSet::from([message.get_author()]);
                last_messages.iter().for_each(|message| {
                    users_set.insert(&message.author);
                });
                let users: Vec<&User> = users_set.into_iter().collect();

                // create dropdown interaction
                let selected_user = Arc::new(Mutex::new(None));
                let _ = message.get_interaction_helper().create_user_dropdown_interaction(
                    embed,
                    users,
                    |value: User| {
                        let selected_user = Arc::clone(&selected_user);
                        Box::pin(
                            async move {
                                let mut selected_user = selected_user.lock().unwrap();
                                *selected_user = Some(value);
                            }
                        )}
                ).await;
                let user = selected_user.lock().unwrap().clone();
                user
            },
            _ => {
                let user = message.get_resolver().resolve_user(mentions[0]).await;
                Some(user.unwrap())
            }
        }
    }

}

impl Command for UserDecorator {

    fn define_usage(&self) -> UsageBuilder {
        self.command.define_usage()
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let target = self.get_target(&params.message).await;
                let augmented_params = params.set_target(target);
                self.command.run(augmented_params.into()).await;
            }
        )
    }

}

pub struct NumberDecorator {
    pub command: Box<dyn Command>,
}

impl NumberDecorator {

    async fn get_number(&self, message: &MessageManager) -> Option<i64> {
        for word in message.words.iter() {
            if let Ok(number) = word.parse::<i64>() {
                return Some(number);
            }
        }

        let embed = MessageManager::create_embed(|embed| {
            embed
                .title("Please provide a number!")
                .description(&format!(
                        "The `{}{}` command requires you to provide a number.\nJust respond in the chat.",
                        message.get_prefix().unwrap(),
                        message.get_command().unwrap()))
        }).await;

        let helper = message.get_interaction_helper();
        let author = message.get_author();
        let reply = helper.await_reply(author, embed.clone()).await;

        // commander never provided a number
        if reply.is_none() {
            return None;
        }

        // commander provided
        let reply = reply.unwrap();
        let number = reply.content.parse::<i64>();

        match number {
            Ok(number) => Some(number),
            Err(_) => {
                let _ = reply.delete(message).await;
                self.invalid_usage(CommandParams::new(message.clone())).await;
                None
            }
        }
    }

}

impl Command for NumberDecorator {

    fn define_usage(&self) -> UsageBuilder {
        self.command.define_usage()
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let number = self.get_number(&params.message).await;
                if number.is_some() {
                    let augmented_params = params.set_number(number);
                    self.command.run(augmented_params.into()).await;
                }
            }
        )
    }

}


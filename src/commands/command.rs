
use serenity::builder::CreateEmbedFooter;
use serenity::model::user::User;
use nonempty::NonEmpty;
use strum_macros::EnumIter;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashSet;

use crate::utility::*;


#[derive(PartialEq, EnumIter)]
pub enum CommandType {
    Casual,
    Games,
    Moderation,
    Tickets
}

impl CommandType {
    pub fn to_string(&self) -> String {
        match self {
            CommandType::Casual => "Casual".to_string(),
            CommandType::Games => "Games".to_string(),
            CommandType::Moderation => "Moderation".to_string(),
            CommandType::Tickets => "Tickets".to_string(),
        }
    }
}

impl From<String> for CommandType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Casual" => CommandType::Casual,
            "Games" => CommandType::Games,
            "Moderation" => CommandType::Moderation,
            "Tickets" => CommandType::Tickets,
            _ => CommandType::Casual,
        }
    }
}

pub struct CommandParams {
    pub message: MessageManager,
    pub target: Option<User>,
    pub number: Option<i64>,
    pub time:   Option<u64>,
}

impl CommandParams {
    pub fn new(message: MessageManager) -> Self {
        Self { message, target: None, number: None, time: None }
    }
    pub fn set_target(mut self, target: Option<User>) -> Self {
        self.target = target;
        self
    }
    pub fn set_number(mut self, number: Option<i64>) -> Self {
        self.number = number;
        self
    }
    pub fn set_time(mut self, time: Option<u64>) -> Self {
        self.time = time;
        self
    }
}

pub trait Command: Send + Sync {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            let muted = &message.get_resolver().resolve_role("Muted").await.unwrap()[0];
            !message.has_role(muted).await
        })
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

    pub async fn get_target(message: &MessageManager) -> Option<User> {
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
                message.get_resolver().resolve_user(mentions[0]).await
            }
        }
    }

}

impl Command for UserDecorator {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        self.command.permission(message)
    }

    fn define_usage(&self) -> UsageBuilder {
        self.command.define_usage()
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let target = UserDecorator::get_target(&params.message).await;
                if target.is_some() {
                    let augmented_params = params.set_target(target);
                    self.command.run(augmented_params.into()).await;
                } else {
                    params.message.reply_failure("User not found.").await;
                }
            }
        )
    }

}

pub struct NumberDecorator {
    pub command: Box<dyn Command>,
}

impl NumberDecorator {

    pub async fn get_number(message: &MessageManager) -> Option<i64> {
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
            Ok(number) => {
                let _ = reply.delete(message).await;
                Some(number)
            },
            Err(_) => None
        }
    }

}

impl Command for NumberDecorator {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        self.command.permission(message)
    }

    fn define_usage(&self) -> UsageBuilder {
        self.command.define_usage()
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let number = NumberDecorator::get_number(&params.message).await;
                if number.is_some() {
                    let augmented_params = params.set_number(number);
                    self.command.run(augmented_params.into()).await;
                } else {
                    params.message.reply_failure("No number provided.").await;
                }
            }
        )
    }

}


pub struct TimeDecorator {
    pub command: Box<dyn Command>,
}

impl TimeDecorator {

    pub async fn get_time(message: &MessageManager) -> Option<u64> {
        for word in message.words.iter() {
            if let Ok(time) = parse_time(word) {
                return Some(time);
            }
        }
        None
    }
}

impl Command for TimeDecorator {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        self.command.permission(message)
    }

    fn define_usage(&self) -> UsageBuilder {
        self.command.define_usage()
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {
                let time = TimeDecorator::get_time(&params.message).await;
                if time.is_some() {
                    let augmented_params = params.set_time(time);
                    self.command.run(augmented_params.into()).await;
                } else {
                    params.message.reply_failure("No time provided.").await;
                }
            }
        )
    }

}



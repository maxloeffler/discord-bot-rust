
use serenity::model::user::User;
use nonempty::NonEmpty;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashSet;

use crate::utility::*;


pub struct CommandParams {
    pub message: MessageManager,
    pub target: Option<User>
}

impl CommandParams {
    pub fn new(message: MessageManager, target: Option<User>) -> Self {
        Self { message, target }
    }
    pub fn set_target(&self, target: Option<User>) -> Self {
        Self { message: self.message.clone(), target }
    }
}

pub enum MatchType {
    Exact,
    Fuzzy(String),
    None
}

pub trait Command: Send + Sync {

    fn is_triggered_by(&self, message: &MessageManager) -> MatchType {
        match message.get_command() {
            Some(word) => {
                let trigger = word.to_lowercase();
                let names = &self.get_names();
                if names.contains(&trigger) {
                    return MatchType::Exact;
                }
                for name in names.iter() {
                    let threshold = name.len() / 3;
                    if string_distance(&trigger, &name) < threshold {
                        return MatchType::Fuzzy(name.to_string());
                    }
                }
                MatchType::None
            },
            None => MatchType::None,
        }
    }

    fn permission<'a>(&'a self, _message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move { true })
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()>;

    fn get_names(&self) -> NonEmpty<String>;

    fn get_usage(&self) -> UsageBuilder {
        UsageBuilder::new(self.get_names().into())
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
                let last_messages = message.get_last_messages(20).await;
                let mut set = HashSet::new();
                last_messages.iter().for_each(|message| {
                    set.insert(&message.author);
                });
                let mut users: Vec<&User> = set.into_iter().collect();
                users.push(message.get_author());

                // create dropdown interaction
                let selected_user = Arc::new(Mutex::new(None));
                let _ = message.create_user_dropdown_interaction(
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

    fn get_names(&self) -> NonEmpty<String> {
        self.command.get_names()
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


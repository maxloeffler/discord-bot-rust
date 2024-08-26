
use serenity::model::prelude::*;
use serenity::all::ComponentInteractionDataKind::StringSelect;
use serenity::model::application::ButtonStyle;
use serenity::builder::{
    CreateEmbed,
    CreateButton,
    CreateInteractionResponse,
    CreateSelectMenu,
    CreateSelectMenuKind,
    CreateSelectMenuOption,
    GetMessages
};

use std::time::Duration;
use std::collections::HashMap;

use crate::utility::*;


pub struct InteractionHelper<'a> {
    channel: ChannelId,
    resolver: &'a Resolver
}

impl<'a> InteractionHelper<'_> {

    pub fn new(channel: ChannelId, resolver: &'a Resolver) -> InteractionHelper<'a> {
        InteractionHelper { channel, resolver }
    }

    pub async fn create_buttons(&self,
                            message: impl ToMessage,
                            mut buttons: Vec<CreateButton>) -> Option<String> {

        // handle button limit
        if buttons.len() > 5 {
            buttons = buttons[..5].to_vec();
            Logger::err(
                &format!(
                    "Discord only supports up to 5 buttons per message ({} requested).",
                    buttons.len()));
        }

        // prepare message
        let message = message.to_message();
        let message = buttons.into_iter().fold(message, |message, button| {
            message.button(button)
        });

        // send message
        let sent_message = self.channel
            .send_message(&self.resolver, message).await.unwrap();

        // await interaction
        let interaction = &sent_message
            .await_component_interaction(&self.resolver.ctx().shard)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if let Some(interaction) = interaction {

            // end interaction
            let _ = interaction.create_response(&self.resolver,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.resolver).await;
            return Some(interaction.data.custom_id.to_string());
        }
        None
    }

    pub async fn create_choice_interaction(&self,
                                     message: impl ToMessage,
                                     yes_callback: BoxedFuture<'a, ()>,
                                     no_callback:  BoxedFuture<'a, ()>) {

        // prepare message
        let yes_button = CreateButton::new("Yes")
            .label("Yes")
            .style(ButtonStyle::Primary);
        let no_button  = CreateButton::new("No")
            .label("No")
            .style(ButtonStyle::Secondary);

        // create interaction and wait for response
        let pressed = self.create_buttons(message, vec![yes_button, no_button]).await;
        if let Some(pressed) = pressed {

            // execute callback
            match pressed.as_str() {
                "Yes" => yes_callback.await,
                "No"  => no_callback.await,
                _ => unreachable!()
            };
        }
    }

    pub async fn create_dropdown_interaction(&self,
                                        message: impl ToMessage,
                                        options: Vec<CreateSelectMenuOption>,
                                        callback: impl FnOnce(&String) -> BoxedFuture<'a, ()>) {

        // prepare message
        let message = message.to_message().select_menu(
            CreateSelectMenu::new("select_menu", CreateSelectMenuKind::String {
                options: options
            })
            .placeholder("Select an option")
        );

        // send message
        let sent_message = self.channel
            .send_message(&self.resolver, message).await.unwrap();

        // await interaction
        let interaction = &sent_message
            .await_component_interaction(&self.resolver.ctx().shard)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if let Some(interaction) = interaction {

            // end interaction
            let _ = interaction.create_response(&self.resolver,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.resolver).await;

            let data = &interaction.data.kind;
            match data {
                StringSelect{values} => {
                    callback(&values[0]).await;
                }
                _ => {}
            }
        }
    }

    pub async fn create_user_dropdown_interaction(&self,
                                        message: impl ToMessage,
                                        users: Vec<&User>,
                                        callback: impl FnOnce(User) -> BoxedFuture<'a, ()>) {

        // prepare message
        let message = message.to_message().select_menu(
            CreateSelectMenu::new("user_select_menu", CreateSelectMenuKind::String {
                options: users.iter().map(|user| {
                    CreateSelectMenuOption::new(self.resolver.resolve_name(user), user.id.to_string())
                        .description(&user.id.to_string())
                }).collect()
            })
            .placeholder("Select a user")
        );

        // send message
        let sent_message = self.channel
            .send_message(&self.resolver, message).await.unwrap();

        // await interaction
        let interaction = sent_message
            .await_component_interaction(&self.resolver.ctx().shard)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if let Some(interaction) = interaction {

            // end interaction
            let _ = interaction.create_response(&self.resolver,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.resolver).await;

            let data = &interaction.data.kind;
            match data {
                StringSelect{values} => {
                    let id = values[0].parse::<u64>().unwrap();
                    let user = self.resolver.resolve_user(UserId::from(id)).await;
                    if user.is_some() {
                        callback(user.unwrap()).await;
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn await_reply(&self, user: &User, message: impl ToMessage) -> Option<Message> {

        let user_id = user.id.to_string();

        // send message
        let sent_message = self.channel
            .send_message(&self.resolver, message.to_message()).await.unwrap();

        // await interaction
        let interaction = self.channel
            .await_reply(&self.resolver.ctx().shard)
            .filter(move |reply| reply.author.id.to_string() == user_id)
            .timeout(Duration::from_secs(60)).await;

        let _ = sent_message.delete(&self.resolver).await;

        // execute callback
        if let Some(interaction) = interaction {
            return Some(interaction);
        }
        None
    }

}

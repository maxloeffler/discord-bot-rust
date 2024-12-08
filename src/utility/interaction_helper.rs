
use serenity::model::prelude::*;
use serenity::all::ComponentInteractionDataKind::StringSelect;
use serenity::model::application::ButtonStyle;
use serenity::builder::{
    CreateEmbed,
    CreateButton,
    CreateActionRow,
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
                            target: UserId,
                            message: impl ToMessage,
                            mut buttons: Vec<CreateButton>) -> Option<String> {

        // handle button limit
        if buttons.len() > 24 {
            buttons = buttons[..24].to_vec();
            Logger::warn(
                &format!(
                    "Discord only supports up to 25 buttons per message ({} requested, 1 is blocked to be the interaction cancel button).",
                    buttons.len()));
        }

        // add cancel button
        let cancel_button = CreateButton::new("cancel")
            .label("Cancel")
            .style(ButtonStyle::Danger);
        buttons.push(cancel_button);

        // split buttons in chunks of 5
        let button_chunks = buttons.chunks(5)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<Vec<CreateButton>>>();
        let action_rows = button_chunks.into_iter().map(|chunk| {
            CreateActionRow::Buttons(chunk)
        }).collect();

        // prepare message
        let message = message.to_message().components(action_rows);

        // send message
        let sent_message = self.channel
            .send_message(&self.resolver, message).await.unwrap();

        // await interaction
        let interaction = &sent_message
            .await_component_interactions(&self.resolver.ctx().shard)
            .author_id(target)
            .timeout(Duration::from_secs(60)).await;

        // execute callback
        if let Some(interaction) = interaction {

            // end interaction
            let _ = interaction.create_response(&self.resolver,
                CreateInteractionResponse::Acknowledge
            ).await;

            // delete message
            let _ = sent_message.delete(&self.resolver).await;
            let id = interaction.data.custom_id.to_string();

            match id.as_str() {
                "cancel" => return None,
                _ => return Some(id)
            }
        }
        None
    }

    // maybe used in the future
    #[allow(unused)]
    pub async fn create_dropdown_interaction(&self,
                                        target: UserId,
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
            .author_id(target)
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
                                        target: UserId,
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
            .author_id(target)
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

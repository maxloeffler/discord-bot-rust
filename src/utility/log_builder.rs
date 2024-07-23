
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};
use serenity::model::user::User;

use crate::utility::*;


#[derive(Clone)]
pub struct LogBuilder<'a> {
    message: &'a MessageManager,
    title: String,
    description: Option<String>,
    color: Option<u64>,
    image: Option<String>,
    target: Option<&'a User>,
    thumbnail: bool,
    fields: Vec<(String, String, bool)>,
}

impl<'a> LogBuilder<'a> {

    pub fn new(message: &MessageManager) -> LogBuilder<'_> {
        LogBuilder {
            message: message,
            title: "No title provided".to_string(),
            description: None,
            color: None,
            image: None,
            target: None,
            thumbnail: true,
            fields: Vec::new(),
        }
    }

    pub async fn build(&self) -> CreateEmbed {
        MessageManager::create_embed(|embed| {
            let author = match &self.target {
                Some(user) => user,
                None => &self.message.get_author()
            };
            let mut embed = embed
                .author(CreateEmbedAuthor::new(self.title.clone())
                    .icon_url(author.face()))
                .fields(self.fields.clone());
            if self.thumbnail {
                embed = embed.thumbnail(author.face());
            }
            if let Some(color) = self.color {
                embed = embed.color(color);
            }
            if let Some(description) = &self.description {
                embed = embed.description(description);
            }
            if let Some(image) = &self.image {
                embed = embed.image(image);
            }
            embed
        }).await
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn color(mut self, color: u64) -> Self {
        self.color = Some(color);
        self
    }

    fn format_user(&self, user: &User) -> String {
        format!("<@{}>", user.id.to_string())
    }

    pub fn user(mut self, user: &User) -> Self {
        self.fields.push(("User".to_string(), self.format_user(user), true));
        self
    }

    pub fn staff(mut self) -> Self {
        let staff = self.message.get_author();
        self.fields.push(("Staff".to_string(), self.format_user(staff), true));
        self
    }

    fn format_timestamp(label: Option<&str>, timestamp: i64) -> String {
        let label = match label {
            Some(label) => format!("{}: ", label),
            None => "".to_string(),
        };
        format!("{}<t:{}> *<t:{}:R>*", label, timestamp, timestamp)
    }

    pub fn timestamp(mut self) -> Self {
        let timestamp = self.message.get_timestamp();
        self.fields.push(("Timestamp".to_string(),
            LogBuilder::format_timestamp(None, timestamp),
            true));
        self
    }

    pub fn labeled_timestamp(mut self, label: impl Into<String>, timestamp: i64) -> Self {
        let label = label.into();
        let timestamp = LogBuilder::format_timestamp(Some(&label), timestamp);
        self.fields.push((label, timestamp, true));
        self
    }

    pub fn channel(mut self) -> Self {
        self.fields.push(("Channel".to_string(),
            format!("<#{}>", self.message.get_channel().get().to_string()),
            true));
        self
    }

    pub fn arbitrary(mut self, label: impl Into<String>, content: impl Into<String>) -> Self {
        self.fields.push((label.into(), content.into(), false));
        self
    }

    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.image = Some(url.into());
        self
    }

    pub fn no_thumbnail(mut self) -> Self {
        self.thumbnail = false;
        self
    }

    pub fn target(mut self, target: &'a User) -> Self {
        self.target = Some(target);
        self
    }

}

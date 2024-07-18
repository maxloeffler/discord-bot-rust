
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};
use serenity::model::user::User;

use crate::utility::*;


#[derive(Clone)]
pub struct LogBuilder {
    message: MessageManager,
    title: String,
    description: Option<String>,
    color: Option<u64>,
    fields: Vec<(String, String, bool)>,
}

impl LogBuilder {

    pub fn new(message: MessageManager) -> LogBuilder {
        LogBuilder {
            message: message,
            title: "No title provided".to_string(),
            description: None,
            color: None,
            fields: Vec::new(),
        }
    }

    pub async fn build(&self) -> CreateEmbed {
        MessageManager::create_embed(|embed| {
            let author = self.message.get_author();
            let embed = embed.clone()
                .author(CreateEmbedAuthor::new(self.title.clone())
                .icon_url(author.face()))
                .fields(self.fields.clone())
                .thumbnail(author.face());
            if let Some(color) = self.color {
                let _ = embed.clone().color(color);
            }
            if let Some(description) = &self.description {
                let _ = embed.clone().description(description);
            }
            embed
        }).await
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn color(mut self, color: u64) -> Self {
        self.color = Some(color);
        self
    }

    fn format_user(&self, user: User) -> String {
        format!("<@{}>", user.id.to_string())
    }

    pub fn user(mut self, user: User) -> Self {
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

    pub fn labeled_timestamp(mut self, label: &str, timestamp: i64) -> Self {
        self.fields.push((label.to_string(),
            LogBuilder::format_timestamp(Some(label), timestamp),
            true));
        self
    }

    pub fn channel(mut self) -> Self {
        self.fields.push(("Channel".to_string(),
            format!("<#{}>", self.message.get_channel().get().to_string()),
            true));
        self
    }

    pub fn arbitrary(mut self, label: &str, content: &str) -> Self {
        self.fields.push((label.to_string(), content.to_string(), false));
        self
    }

}

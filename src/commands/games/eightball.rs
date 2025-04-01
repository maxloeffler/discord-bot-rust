
use serenity::builder::CreateEmbedFooter;
use serenity::all::UserId;
use rand::seq::IndexedRandom;
use nonempty::{NonEmpty, nonempty};

use crate::commands::command::*;
use crate::utility::*;
use crate::databases::*;


pub struct EightBallCommand;

impl EightBallCommand {

    pub const OPTIONS: [&'static str; 13] = [
        "It is certain.",
        "It is decidedly so.",
        "Without a doubt.",
        "Yes definitely.",
        "You may rely on it.",
        "Most likely.",
        "As I see it, yes.",
        "Very doubtful.",
        "My reply is no.",
        "Don't count on it.",
        "Outlook not so good.",
        "Ask again later.",
        "Better not tell you now.",
    ];

    fn random_option() -> &'static str {
        EightBallCommand::OPTIONS.choose(&mut rand::rng()).unwrap()
    }

}

impl Command for EightBallCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(
            CommandType::Games,
            nonempty!["8ball".to_string(), "eightball".to_string()]
        )
            .add_required("question")
            .example("Will I get a promotion to moderator next week?")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let content = message.payload(None, None);

                if content.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                // resolve bot
                let bot_id: UserId = ConfigDB::get_instance()
                    .get("bot_id").await.unwrap().into();
                let bot = message.get_resolver().resolve_user(bot_id).await.unwrap();

                // create embed
                let option = EightBallCommand::random_option();
                let embed = message.get_log_builder()
                    .title("Magic 🎱")
                    .target(&bot)
                    .no_thumbnail()
                    .arbitrary_block("Question", content)
                    .arbitrary_block("Answer", option)
                    .build().await
                    .footer(CreateEmbedFooter::new(
                        format!("Question by {}", message.resolve_name()))
                        .icon_url(message.get_author().face()));

                let _ = message.reply(embed).await;
            }
        )
    }

}


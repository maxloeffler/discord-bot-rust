
use serenity::all::ChannelId;
use rand::seq::SliceRandom;
use nonempty::{NonEmpty, nonempty};

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct VerifyCommand;

impl VerifyCommand {

    pub const WELCOME_MESSAGES: [&'static str; 11] = [
        "Please help them with roles if they need help.",
        "Welcome to Kalopsia! You can ask for help by pinging a staff member.",
        "Thank you for joining Kalopsia! If you'd like help then you can always ask anyone chat!",
        "Welcome to Kalopsia! We're thrilled that you chose to join!",
        "Please help them with roles if they want! \nWe hope you enjoy the server!",
        "https://tenor.com/view/the-god-father-marlon-brando-vito-corleone-talk-to-me-gif-20107028",
        "https://tenor.com/view/star-wars-baby-yoda-the-mandalorian-welcome-wave-gif-16179355",
        "https://tenor.com/view/welcome-captain-gif-18905364",
        "https://tenor.com/view/welcome-gif-18737601",
        "https://giphy.com/gifs/welcome-3oEjHQn7PBRvy9A5mE",
        "https://giphy.com/gifs/welcome-austin-powers-dr-evil-l0MYC0LajbaPoEADu"
    ];

    fn random_welcome_message() -> &'static str {
        VerifyCommand::WELCOME_MESSAGES.choose(&mut rand::thread_rng()).unwrap()
    }

}

impl Command for VerifyCommand {

    fn get_names(&self) -> NonEmpty<String> {
        nonempty!["verify".to_string()]
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let channel_verify = ConfigDB::get_instance().lock().await
                    .get("channel_verify").await.unwrap().to_string();

                if message.get_channel().to_string() == channel_verify {

                    let roles = &message.resolve_role(vec!["User Restriction", "Kalopsian"]).await.unwrap();

                    if !message.has_role(&roles[0]).await || message.has_role(&roles[1]).await {
                        message.reply_failure("You are already verified!").await;
                        return;
                    }

                    let member = message.resolve_member().await;
                    if let Some(member) = member {

                        // setup roles
                        let _ = member.remove_role(message, &roles[0]).await;
                        let _ = member.add_role(message, &roles[1]).await;

                        message.reply_success().await;

                        // send welcome message
                        let channel_welcome = ConfigDB::get_instance().lock().await
                            .get("channel_welcome").await.unwrap().to_string();
                        let channel = ChannelId::from_str(&channel_welcome).unwrap();
                        let welcome_message = VerifyCommand::random_welcome_message();
                        let _ = channel.send_message(message,
                            format!(
                                "<@{}> has joined Kalopsia!\n{}",
                                message.get_author().id,
                                welcome_message).to_message()
                            ).await;
                    }
                }
            }
        )
    }

}


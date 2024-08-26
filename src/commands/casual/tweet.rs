
use serenity::all::*;
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};
use chrono::Utc;

use std::str::FromStr;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct TweetCommand;

impl Command for TweetCommand {

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "tweet".to_string(),
        ])
            .add_required("message (max 280 characters)")
            .example("Twitter is now X!")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let author = &message.get_author().id.to_string();
                let content = message.payload(None, None);

                if content.is_empty() {
                    self.invalid_usage(params).await;
                    return;
                }

                if content.len() > 280 {
                    self.invalid_usage(params).await;
                    return;
                }

                let last_tweet = TweetsDB::get_instance().lock().await
                    .get_last(author, 1).await.unwrap();

                // You can only tweet every 10 minutes to rate limit pings
                if let Some(tweet) = last_tweet.first() {
                    let now = chrono::Utc::now().timestamp();
                    let next_tweet = tweet.timestamp + 10 * 60;
                    if now < next_tweet {
                        let _ = message.reply_failure(
                            &format!("You can tweet again <t:{}:R>", next_tweet)).await;
                        return;
                    }
                }

                // resolve role and channel
                let role_tweets = message.get_resolver().resolve_role("Tweets").await.unwrap()[0].id;
                let channel_tweets: ChannelId = ConfigDB::get_instance().lock().await
                    .get("channel_tweets").await.unwrap().into();

                // create tweet message
                let allowed_mentions = CreateAllowedMentions::new()
                    .roles(vec![role_tweets]);
                let reactions = vec![ReactionType::Unicode("👍".to_string()),
                                     ReactionType::Unicode("👎".to_string())];
                let tweet = CreateMessage::new()
                    .content(format!("<@{}> **tweeted!** <@&{}>\n\n{}", author, role_tweets.to_string(), content))
                    .reactions(reactions)
                    .allowed_mentions(allowed_mentions);

                // log last tweet
                TweetsDB::get_instance().lock().await
                    .set(author, &content).await;

                // send to tweets channel
                let _ = channel_tweets.send_message(&message, tweet).await;

                let _ = message.reply_success().await;
            }
        )
    }
}




use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct FlagsCommand;

impl Command for FlagsCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "flags".to_string(),
        ])
            .add_required("user")
            .add_optional("-more")
            .example("flags @BadBoy -more")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                let flags = FlagsDB::get_instance().lock().await
                    .get_all(&target.id.to_string()).await;

                if let Ok(mut flags) = flags {

                    // get number of total flags
                    let total_flags = flags.len();

                    // get amount of flags to display
                    let limit = min(total_flags,
                        match message.has_parameter("more") {
                            true => 24,
                            false => 5
                        });
                    let flags = flags.split_off(total_flags - limit);

                    // create embed
                    let name = message.get_resolver().resolve_name(target);
                    let mut builder = message.get_log_builder()
                        .target(target)
                        .title(format!("{}'s Flags", name))
                        .no_thumbnail();

                    // add flags to embed
                    let embed = match flags.len() {
                        0 => builder.description("No registered flags.").build().await,
                        len @ _ => {
                            for flag in flags.into_iter() {
                                builder = builder.mod_log(&flag);
                            }
                            builder.build().await
                                .footer(CreateEmbedFooter::new(
                                    format!("Displaying {} of {} Flags", len, total_flags)
                                ))
                        }
                    };

                    let _ = message.reply(embed).await;
                }
            }
        )
    }

}



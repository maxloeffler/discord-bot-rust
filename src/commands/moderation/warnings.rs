
use serenity::builder::CreateEmbedFooter;
use nonempty::{NonEmpty, nonempty};

use std::cmp::min;
use std::sync::Arc;

use crate::commands::command::{Command, CommandParams};
use crate::utility::*;
use crate::databases::*;


pub struct WarningsCommand;

impl Command for WarningsCommand {

    fn permission<'a>(&'a self, message: &'a MessageManager) -> BoxedFuture<'_, bool> {
        Box::pin(async move {
            message.is_trial().await
        })
    }

    fn define_usage(&self) -> UsageBuilder {
        UsageBuilder::new(nonempty![
            "warnings".to_string(),
        ])
            .add_required("user")
            .add_optional("more")
            .example("@BadBoy")
    }

    fn run(&self, params: CommandParams) -> BoxedFuture<'_, ()> {
        Box::pin(
            async move {

                let message = &params.message;
                let target = &params.target.unwrap();

                let warnings = WarningsDB::get_instance()
                    .get_all(&target.id.to_string()).await;

                if let Ok(mut warnings) = warnings {

                    // get number of total warnings
                    let total_warnings = warnings.len();

                    // get amount of warnings to display
                    let limit = min(total_warnings,
                        match message.has_parameter("more") {
                            true => 24,
                            false => 5
                        });
                    let warnings = warnings.split_off(total_warnings - limit);

                    // create embed
                    let name = message.get_resolver().resolve_name(target);
                    let mut builder = message.get_log_builder()
                        .target(target)
                        .title(format!("{}'s Warnings", name))
                        .no_thumbnail();

                    // add warnings to embed
                    let embed = match warnings.len() {
                        0 => builder.description("No registered warnings.").build().await,
                        len @ _ => {
                            for warning in warnings.into_iter() {
                                builder = builder.mod_log(&warning);
                            }
                            builder.build().await
                                .footer(CreateEmbedFooter::new(
                                    format!("Displaying {} of {} Warnings", len, total_warnings)
                                ))
                        }
                    };

                    let _ = message.reply(embed).await;

                    #[cfg(feature = "auto_moderation")]
                    AutoModerator::get_instance()
                        .check_warnings(message, target).await;
                }
            }
        )
    }

}



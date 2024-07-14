
use nonempty::NonEmpty;

use crate::utility::message_manager::MessageManager;
use crate::utility::usage_builder::UsageBuilder;
use crate::utility::mixed::{BoxedFuture, string_distance};


pub enum MatchType {
    Exact,
    Fuzzy(String),
    None
}

pub trait Command: Send + Sync {

    fn is_triggered_by(&self, message: MessageManager) -> MatchType {
        let trigger = message.get_command();
        match trigger {
            Some(word) => {
                let trigger = word.to_lowercase();
                for name in self.get_names().iter() {
                    let threshold = name.len() / 3;
                    if trigger.eq(name) {
                        return MatchType::Exact;
                    }
                    if string_distance(&trigger, &name) < threshold {
                        return MatchType::Fuzzy(name.to_string());
                    }
                }
                MatchType::None
            },
            None => MatchType::None,
        }
    }

    fn permission(&self) -> bool {
        true
    }

    fn run(&self, message: MessageManager) -> BoxedFuture<'_>;

    fn get_names(&self) -> NonEmpty<String>;

    fn get_usage(&self) -> UsageBuilder {
        UsageBuilder::new(self.get_names().into())
    }

}


use nonempty::NonEmpty;

use std::pin::Pin;
use std::future::Future;

use crate::utility::message_manager::MessageManager;
use crate::utility::utility_builder::UsageBuilder;


pub type BoxedFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Command: Send + Sync {

    fn is_triggered_by(&self, word: String) -> bool {
        let word = word.to_lowercase();
        self.get_names().contains(&word)
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

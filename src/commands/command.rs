
use std::pin::Pin;
use std::future::Future;

use crate::utility::message_manager::MessageManager;


pub type BoxedFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait Command: Send + Sync {

    fn get_names(&self) -> Vec<String>;

    fn woke_by(&self, word: String) -> bool {
        let word = word.to_lowercase();
        self.get_names().contains(&word)
    }

    fn permission(&self) -> bool {
        true
    }

    fn run(&self, message: MessageManager) -> BoxedFuture<'_>;

}


use std::pin::Pin;
use std::future::Future;


pub type BoxedFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub fn string_distance(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).filter(|(a, b)| a != b).count()
}

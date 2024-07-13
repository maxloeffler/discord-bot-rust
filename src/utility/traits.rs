
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use std::sync::Arc;

use crate::utility::database::Database;


pub trait Singleton: Sized {
    fn get_instance() -> &'static Mutex<Self>;
    fn new() -> Self;
}

macro_rules! impl_singleton {
    ($t:ty) => {
        impl Singleton for $t {
            fn get_instance() -> &'static Mutex<Self> {
                static INSTANCE: Lazy<Arc<Mutex<$t>>> = Lazy::new(|| Arc::new(Mutex::new(<$t>::new())));
                &INSTANCE
            }

            fn new() -> Self {
                <$t>::new()
            }
        }
    };
}

impl_singleton!(Database);


pub trait ToList<T: ?Sized> {
    fn to_list(&self) -> Vec<T> where T: Clone;
}

impl<T> ToList<T> for T {
    fn to_list(&self) -> Vec<T> where T: Clone {
        vec![self.clone()]
    }
}

impl<T> ToList<T> for Vec<T> {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.clone()
    }
}

impl<T> ToList<T> for [T] {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.iter().map(|s| s.clone()).collect()
    }
}

impl<T> ToList<T> for &T {
    fn to_list(&self) -> Vec<T> where T: Clone {
        vec![(*self).clone()]
    }
}

impl<T> ToList<T> for Vec<&T> {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.iter().map(|s| (*s).clone()).collect()
    }
}

impl<T> ToList<T> for &[T] {
    fn to_list(&self) -> Vec<T> where T: Clone {
        self.iter().map(|s| s.clone()).collect()
    }
}

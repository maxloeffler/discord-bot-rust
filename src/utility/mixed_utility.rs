

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

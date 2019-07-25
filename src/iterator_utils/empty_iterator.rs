use std::marker::PhantomData;

#[derive(Debug)]
pub(in crate) struct EmptyIterator<T>(PhantomData<T>);

impl<T> EmptyIterator<T> {
    pub(in crate) const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for EmptyIterator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Iterator for EmptyIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::EmptyIterator;

    #[test]
    fn iterator_collects_to_empty_vector() {
        assert_eq!(EmptyIterator::<u32>::new().collect::<Vec<_>>(), Vec::<u32>::new());
    }
}

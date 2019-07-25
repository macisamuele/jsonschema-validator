use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

#[allow(dead_code)] // GeneratorIterator not yet used in the codebase (other than tests)
#[allow(missing_debug_implementations)] // No debug implementation to avoid to consume the generator
pub(in crate) struct GeneratorIterator<Y>(Box<dyn Generator<Yield = Y, Return = ()>>);

impl<G, Y> From<G> for GeneratorIterator<Y>
where
    G: 'static + Generator<Yield = Y, Return = ()>,
{
    fn from(generator: G) -> Self {
        Self(Box::new(generator))
    }
}

impl<Y> Iterator for GeneratorIterator<Y> {
    type Item = Y;

    fn next(&mut self) -> Option<Self::Item> {
        let pinned_generator = {
            #[allow(unsafe_code)]
            unsafe {
                Pin::new_unchecked(&mut *self.0)
            }
        };
        if let GeneratorState::Yielded(value) = pinned_generator.resume(()) {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GeneratorIterator;

    #[test]
    fn generator_is_converted_to_iterator() {
        let iterator: Box<dyn Iterator<Item = u32>> = Box::new(GeneratorIterator::from(|| {
            yield 3;
            yield 4;
            yield 5;
        }));
        assert_eq!(iterator.collect::<Vec<_>>(), vec![3, 4, 5]);
    }

    #[test]
    fn generator_behaves_as_iterator() {
        let mut iterator = GeneratorIterator::from(|| {
            yield 3;
            yield 4;
            yield 5;
        });
        assert_eq!(iterator.next(), Some(3));
        assert_eq!(iterator.collect::<Vec<_>>(), vec![4, 5]);
    }
}

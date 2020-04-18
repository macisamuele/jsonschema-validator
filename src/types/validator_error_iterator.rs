use crate::types::validation_error::ValidationError;
use std::ops::{Deref, DerefMut};

#[allow(missing_debug_implementations)] // No debug implementation to avoid to consume the iterator
pub(in crate) struct ValidationErrorIterator(Box<dyn Iterator<Item = ValidationError>>);

impl Iterator for ValidationErrorIterator {
    type Item = ValidationError;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Deref for ValidationErrorIterator {
    type Target = dyn Iterator<Item = ValidationError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ValidationErrorIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ValidationError> for ValidationErrorIterator {
    fn from(value: ValidationError) -> Self {
        Self(Box::new(vec![value].into_iter()))
    }
}

impl ValidationErrorIterator {
    pub(in crate) fn new<I: 'static + IntoIterator<Item = ValidationError>>(value: I) -> Self {
        Self(Box::new(value.into_iter()))
    }
}

use crate::{
    iterator_utils::empty_iterator::EmptyIterator,
    types::{draft_version::DraftVersion, schema::Schema, validator_error_iterator::ValidationErrorIterator},
};
use json_trait_rs::JsonType;
use std::{collections::HashMap, sync::Arc};
use url::Url;

#[derive(Debug)]
pub(in crate) struct Scope {
    pub(in crate) draft_version: DraftVersion,
    // TODO: Verify if we need a thread-safe cache or this is good enough
    pub(in crate) schema_cache: HashMap<Url, Arc<Schema>>,
}

impl Scope {
    pub(in crate) fn validation_errors<T: 'static + JsonType>(&self, schema_url: &Url, value: &T) -> ValidationErrorIterator {
        match &self.schema_cache.get(schema_url) {
            Some(arc_schema) => arc_schema.validation_errors("", value),
            None => ValidationErrorIterator::new(EmptyIterator::new()),
        }
    }

    pub(in crate) fn is_valid<T: 'static + JsonType>(&self, path: &Url, value: &T) -> bool {
        self.validation_errors(path, value).peekable().peek().is_none()
    }
}

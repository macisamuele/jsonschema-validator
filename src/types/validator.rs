use crate::types::{keyword_type::KeywordType, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validator_error_iterator::ValidationErrorIterator};
use json_trait_rs::JsonType;
use std::{any::Any, fmt::Debug};

pub(in crate) trait Validator: Debug + Sync + Send {
    fn compile<T: 'static + JsonType>(scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Option<Self>, SchemaError>
    where
        Self: Sized;

    fn keyword_type(&self) -> KeywordType;

    fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator;

    fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.validation_errors(path, value).next().is_none()
    }

    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
use crate::types::keyword_type::KeywordType;
use crate::types::{draft_version::DraftVersion, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validator_error_iterator::ValidationErrorIterator};
use json_trait_rs::JsonType;

#[allow(clippy::empty_enum)]
#[derive(Debug)]
pub(in crate) enum DraftValidator {}

impl DraftValidator {
    #[allow(clippy::unused_self, unused_variables)]
    pub(in crate) fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        unimplemented!()
    }

    pub(in crate) fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.validation_errors(path, value).next().is_none()
    }

    #[allow(clippy::unused_self)]
    #[cfg(test)]
    pub(in crate) fn keyword_type(&self) -> KeywordType {
        unimplemented!()
    }
}

#[allow(unused_variables)]
pub(in crate) fn compile_draft_validators<T: 'static + JsonType>(scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Vec<DraftValidator>, SchemaError> {
    let mut validators: Vec<DraftValidator> = Vec::new();

    match scope_builder.draft_version {
        DraftVersion::Draft4 => {}
    };
    validators.shrink_to_fit();
    Ok(validators)
}

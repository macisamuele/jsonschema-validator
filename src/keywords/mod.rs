pub(in crate) mod type_;

#[cfg(test)]
use crate::types::keyword_type::KeywordType;
use crate::types::{
    draft_version::DraftVersion, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validator::Validator, validator_error_iterator::ValidationErrorIterator,
};
use json_trait_rs::JsonType;

#[derive(Debug)]
pub(in crate) enum DraftValidator {
    Type(type_::Type),
}

impl DraftValidator {
    pub(in crate) fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        match self {
            Self::Type(validator) => validator.validation_errors(path, value),
        }
    }

    pub(in crate) fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.validation_errors(path, value).next().is_none()
    }

    #[cfg(test)]
    pub(in crate) fn keyword_type(&self) -> KeywordType {
        match self {
            Self::Type(validator) => validator.keyword_type(),
        }
    }
}

pub(in crate) fn compile_draft_validators<T: 'static + JsonType>(scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Vec<DraftValidator>, SchemaError> {
    let mut validators: Vec<DraftValidator> = Vec::new();

    match scope_builder.draft_version {
        DraftVersion::Draft4 => {
            if let Some(validator) = type_::Type::compile(scope_builder, schema)? {
                validators.push(DraftValidator::Type(validator));
            }
        }
    };
    validators.shrink_to_fit();
    Ok(validators)
}

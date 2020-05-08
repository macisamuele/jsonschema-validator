pub(in crate) mod properties_;
pub(in crate) mod ref_;
pub(in crate) mod required_;
pub(in crate) mod type_;

#[cfg(test)]
use crate::types::keyword_type::KeywordType;
use crate::types::{
    draft_version::DraftVersion, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validator::Validator, validator_error_iterator::ValidationErrorIterator,
};
use json_trait_rs::JsonType;

#[derive(Debug)]
pub(in crate) enum DraftValidator {
    Properties(properties_::Properties),
    Ref(ref_::Ref),
    Required(required_::Required),
    Type(type_::Type),
}

impl DraftValidator {
    pub(in crate) fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        match self {
            Self::Properties(validator) => validator.validation_errors(path, value),
            Self::Ref(validator) => validator.validation_errors(path, value),
            Self::Required(validator) => validator.validation_errors(path, value),
            Self::Type(validator) => validator.validation_errors(path, value),
        }
    }

    pub(in crate) fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.validation_errors(path, value).next().is_none()
    }

    #[cfg(test)]
    pub(in crate) fn keyword_type(&self) -> KeywordType {
        match self {
            Self::Properties(validator) => validator.keyword_type(),
            Self::Ref(validator) => validator.keyword_type(),
            Self::Required(validator) => validator.keyword_type(),
            Self::Type(validator) => validator.keyword_type(),
        }
    }
}

pub(in crate) fn compile_draft_validators<T: 'static + JsonType>(scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Vec<DraftValidator>, SchemaError> {
    let mut validators: Vec<DraftValidator> = Vec::new();

    match scope_builder.draft_version {
        DraftVersion::Draft4 => {
            if let Some(validator) = properties_::Properties::compile(scope_builder, schema)? {
                validators.push(DraftValidator::Properties(validator));
            }
            if let Some(validator) = ref_::Ref::compile(scope_builder, schema)? {
                validators.push(DraftValidator::Ref(validator));
            }
            if let Some(validator) = required_::Required::compile(scope_builder, schema)? {
                validators.push(DraftValidator::Required(validator));
            }
            if let Some(validator) = type_::Type::compile(scope_builder, schema)? {
                validators.push(DraftValidator::Type(validator));
            }
        }
    };
    validators.shrink_to_fit();
    Ok(validators)
}

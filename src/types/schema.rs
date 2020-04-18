use crate::{
    keywords::DraftValidator,
    types::{
        draft_version::DraftVersion, keyword_type::KeywordType, schema_error::SchemaError, scope_builder::ScopeBuilder, validation_error::ValidationError,
        validator_error_iterator::ValidationErrorIterator,
    },
};
use json_trait_rs::{JsonType, PrimitiveType, RustType};
use std::{ops::Deref, sync::Arc};
use url::Url;

#[derive(Debug)]
pub(in crate) struct Schema {
    pub(in crate) draft_version: DraftVersion,
    pub(in crate) validators: Vec<DraftValidator>,
    pub(in crate) path: Url,
    pub(in crate) raw_schema: Arc<RustType>,
    is_initialised: bool,
}

impl Deref for Schema {
    type Target = RustType;

    fn deref(&self) -> &Self::Target {
        &*self.raw_schema
    }
}

impl Schema {
    pub(in crate) fn create<T, J>(scope_builder: &mut ScopeBuilder<T>, path: &Url, raw_schema: &J) -> Result<Self, SchemaError>
    where
        T: 'static + JsonType,
        J: JsonType,
    {
        let raw_schema_rust_type = raw_schema.to_rust_type();
        if raw_schema.is_object() {
            let draft_version = scope_builder.draft_version;
            Ok(Self {
                draft_version,
                path: path.clone(),
                validators: Vec::with_capacity(0),
                raw_schema: Arc::new(raw_schema_rust_type),
                is_initialised: false,
            })
        } else {
            Err(SchemaError::Malformed {
                path: path.clone(),
                keyword: KeywordType::Unknown,
                detail: format!(
                    "raw_schema type is {} while is expected type {}. raw_schema: {}",
                    raw_schema.primitive_type(),
                    PrimitiveType::Object,
                    raw_schema_rust_type
                ),
            })
        }
    }

    pub(in crate) fn initialise(&mut self) {
        if !self.is_initialised {
            self.is_initialised = true;
            // TODO: initialise validators if needed
        }
    }

    pub(in crate) fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        if self.is_initialised {
            // TODO: Find a way to avoid the collection of the validation errors (after all we are returning an iterator)
            ValidationErrorIterator::new(self.validators.iter().flat_map(|validator| validator.validation_errors(path, value)).collect::<Vec<_>>())
        } else {
            ValidationErrorIterator::from(ValidationError::new(path, KeywordType::Unknown, "Uninitialised schema"))
        }
    }

    pub(in crate) fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.validators.iter().all(|validator| validator.is_valid(path, value))
    }

    #[cfg(test)]
    pub(in crate) fn do_on_validator<R>(&self, keyword_type: KeywordType, closure: &dyn Fn(Option<&DraftValidator>) -> R) -> R {
        for validator in &self.validators {
            if validator.keyword_type() == keyword_type {
                return closure(Some(validator));
            }
        }
        closure(None as Option<&DraftValidator>)
    }
}

#[cfg(test)]
pub(in crate) fn draft4_schema(raw_schema: RustType) -> Result<Schema, SchemaError> {
    crate::types::scope_builder::scope_builder_create_and_build(DraftVersion::Draft4, raw_schema, &|scope_builder, generated_url, raw_schema| {
        Schema::create(scope_builder, generated_url, raw_schema)
    })
}

#[cfg(test)]
mod tests {
    use super::{draft4_schema, Schema};
    use crate::types::{draft_version::DraftVersion, keyword_type::KeywordType, schema_error::SchemaError, scope_builder::scope_builder_create, validation_error::ValidationError};
    use json_trait_rs::{rust_type, PrimitiveType, RustType};
    use test_case::test_case;

    #[test_case(&rust_type!(null), PrimitiveType::Null)]
    #[test_case(&rust_type!(false), PrimitiveType::Boolean)]
    #[test_case(&rust_type!(1), PrimitiveType::Integer)]
    #[test_case(&rust_type!("2"), PrimitiveType::String)]
    #[test_case(&rust_type!([3, 4, 5]), PrimitiveType::Array)]
    fn build_from_invalid_type_schema(raw_schema: &RustType, expected_primitive_type_in_error: PrimitiveType) {
        assert!(matches!(
            draft4_schema(raw_schema.clone()),
            Err(SchemaError::Malformed {
                keyword: KeywordType::Unknown,
                detail,
                ..
            }) if detail == format!("raw_schema type is {} while is expected type {}. raw_schema: {}", expected_primitive_type_in_error, PrimitiveType::Object, raw_schema)
        ));
    }

    fn validate_unbuilt_schema() {
        let unbuilt_schema = scope_builder_create(DraftVersion::Draft4, rust_type!({}), &|scope_builder, generated_url, raw_schema| {
            Schema::create(scope_builder, generated_url, raw_schema)
        })
        .1
        .unwrap();
        assert_eq!(
            unbuilt_schema.validation_errors("#", &rust_type!({"bool": true})).collect::<Vec<_>>(),
            vec![ValidationError::new("#/bool", KeywordType::Unknown, "Uninitialised schema")],
        );
    }
}

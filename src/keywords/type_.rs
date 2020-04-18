use crate::{
    iterator_utils::empty_iterator::EmptyIterator,
    types::{
        keyword_type::KeywordType, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validation_error::ValidationError, validator::Validator,
        validator_error_iterator::ValidationErrorIterator,
    },
};
use json_trait_rs::{Error, JsonType, PrimitiveType};
use std::{any::Any, collections::HashSet, convert::TryFrom, fmt::Debug};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub(in crate) struct Type {
    pub(in crate) types: HashSet<PrimitiveType>,
}

#[allow(unsafe_code)]
unsafe impl Sync for Type {}
#[allow(unsafe_code)]
unsafe impl Send for Type {}

fn malformed_error(path: &Url, error: &Error) -> SchemaError {
    SchemaError::Malformed {
        path: path.clone(),
        keyword: KeywordType::Type,
        detail: format!("{}", error),
    }
}

fn into_str<'l, S: Into<&'l str>>(value: S) -> &'l str {
    value.into()
}

fn malformed_type<J: JsonType>(path: &Url, value: &J) -> SchemaError {
    SchemaError::Malformed {
        path: path.clone(),
        keyword: KeywordType::Type,
        detail: format!(
            "Values have to be of `{}` type. {:?} has type `{}`.",
            into_str(PrimitiveType::String),
            value,
            into_str(value.primitive_type())
        ),
    }
}

impl Validator for Type {
    fn compile<T: 'static + JsonType>(_scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Option<Self>, SchemaError>
    where
        Self: Sized,
    {
        let type_attribute = if let Some(value) = schema.get_attribute("type") {
            value
        } else {
            // type attribute is not there so we're done here
            return Ok(None);
        };

        let mut types = HashSet::new();
        if let Some(type_str) = type_attribute.as_string() {
            let primitive_type = match PrimitiveType::try_from(type_str) {
                Ok(value) => value,
                Err(ref error) => {
                    return Err(malformed_error(&schema.path, error));
                }
            };
            let _ = types.insert(primitive_type);
        } else if let Some(type_array) = type_attribute.as_array() {
            if type_array.is_empty() {
                return Err(SchemaError::Malformed {
                    path: schema.path.clone(),
                    keyword: KeywordType::Type,
                    detail: "This array MUST have at least one element.".to_string(),
                });
            }

            for array_item in type_array {
                if let Some(type_str) = array_item.as_string() {
                    let primitive_type = match PrimitiveType::try_from(type_str) {
                        Ok(value) => value,
                        Err(ref error) => {
                            return Err(malformed_error(&schema.path, error));
                        }
                    };
                    let _ = types.insert(primitive_type);
                } else {
                    return Err(malformed_type(&schema.path, array_item));
                }
            }
        } else {
            return Err(malformed_type(&schema.path, type_attribute));
        }

        Ok(Some(Self { types }))
    }

    fn keyword_type(&self) -> KeywordType {
        KeywordType::Type
    }

    fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        let value_primitive_type = value.primitive_type();

        if self.types.iter().all(|primitive_type| primitive_type != &value_primitive_type) {
            ValidationErrorIterator::from(ValidationError::new(path, KeywordType::Type, "Invalid Type"))
        } else {
            ValidationErrorIterator::new(EmptyIterator::new())
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod validator_tests {
    use super::Type;
    use crate::{
        hash_set,
        keywords::DraftValidator,
        types::{
            keyword_type::KeywordType,
            schema::{draft4_schema, Schema},
            schema_error::SchemaError,
            validation_error::ValidationError,
        },
    };
    use json_trait_rs::{rust_type, PrimitiveType, RustType};
    use std::collections::HashSet;
    use test_case::test_case;

    // Testing constants
    fn string_schema() -> Schema {
        draft4_schema(rust_type!({"type": "string"})).expect("Schema is supposed to be valid")
    }

    fn string_or_bool_schema() -> Schema {
        draft4_schema(rust_type!({"type": ["boolean", "string"]})).expect("Schema is supposed to be valid")
    }

    // Tests
    #[test_case(rust_type!({"type": "string"}), &hash_set![PrimitiveType::String])]
    #[test_case(rust_type!({"type": "integer"}), &hash_set![PrimitiveType::Integer])]
    #[test_case(rust_type!({"type": ["string", "integer"]}), &hash_set![PrimitiveType::String, PrimitiveType::Integer])]
    fn build_type_object_valid(raw_schema: RustType, expected_types: &HashSet<PrimitiveType>) {
        let schema = draft4_schema(raw_schema).expect("Schema is supposed to be valid");
        schema.do_on_validator(KeywordType::Type, &|maybe_draft_validator| {
            let draft_validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert!(matches!(
                draft_validator,
                DraftValidator::Type(Type { types, ..})
                if types == expected_types
            ))
        });
    }

    #[test_case(
        rust_type!({"type": "not-real-type"}),
        "Unsupported primitive type `not-real-type`. Available types are defined by `json_trait_rs::PrimitiveType::VARIANTS`"
    )]
    #[test_case(
        rust_type!({"type": []}),
        "This array MUST have at least one element."
    )]
    #[test_case(
        rust_type!({"type": ["not-real-type-in-array"]}),
        "Unsupported primitive type `not-real-type-in-array`. Available types are defined by `json_trait_rs::PrimitiveType::VARIANTS`"
    )]
    #[test_case(
        rust_type!({"type": [1]}),
        "Values have to be of `string` type. Integer(1) has type `integer`."
    )]
    #[test_case(
        rust_type!({"type": 2}),
        "Values have to be of `string` type. Integer(2) has type `integer`."
    )]
    fn build_type_object_invalid(raw_schema: RustType, expected_malformed_error_detail: &str) {
        assert!(matches!(
            draft4_schema(raw_schema),
            Err(SchemaError::Malformed {
                keyword: KeywordType::Type,
                detail,
                ..
            }) if detail == expected_malformed_error_detail
        ));
    }

    #[test_case(&string_schema(), &rust_type!("text"), &[])]
    #[test_case(&string_schema(), &rust_type!(true), &[ValidationError::new("#/", KeywordType::Type, "Invalid Type")])]
    #[test_case(&string_schema(), &rust_type!(1), &[ValidationError::new("#/", KeywordType::Type, "Invalid Type")])]
    #[test_case(&string_or_bool_schema(), &rust_type!("text"), &[])]
    #[test_case(&string_or_bool_schema(), &rust_type!(true), &[])]
    #[test_case(&string_or_bool_schema(), &rust_type!(1), &[ValidationError::new("#/", KeywordType::Type, "Invalid Type")])]
    fn validate(schema: &Schema, object: &RustType, expected_validation_errors: &[ValidationError]) {
        schema.do_on_validator(KeywordType::Type, &|maybe_draft_validator| {
            let validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert_eq!(validator.validation_errors("#", object).collect::<Vec<_>>(), expected_validation_errors);
        });
    }
}

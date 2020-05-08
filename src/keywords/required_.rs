use crate::{
    iterator_utils::empty_iterator::EmptyIterator,
    types::{
        keyword_type::KeywordType, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validation_error::ValidationError, validator::Validator,
        validator_error_iterator::ValidationErrorIterator,
    },
};
use json_trait_rs::{JsonMapTrait, JsonType, PrimitiveType};
use std::{any::Any, collections::HashSet, fmt::Debug};

#[derive(Debug, Clone)]
pub(in crate) struct Required {
    pub(in crate) required_attributes: HashSet<String>,
}

#[allow(unsafe_code)]
unsafe impl Sync for Required {}
#[allow(unsafe_code)]
unsafe impl Send for Required {}

impl Validator for Required {
    fn compile<T: 'static + JsonType>(_scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Option<Self>, SchemaError>
    where
        Self: Sized,
    {
        let required_attribute = if let Some(value) = schema.get_attribute("required") {
            value
        } else {
            // required attribute is not there so we're done here
            return Ok(None);
        };

        let required_array = if let Some(value) = required_attribute.as_array() {
            value
        } else {
            // required attribute is not of type array. So we should not consider it
            return Err(SchemaError::Malformed {
                path: schema.path.clone(),
                keyword: KeywordType::Required,
                detail: format!(
                    "Required attribute must be of type {} (found type {})",
                    PrimitiveType::Array,
                    required_attribute.primitive_type()
                ),
            });
        };

        let mut required_attributes = HashSet::new();

        for (index, item) in required_array.enumerate() {
            if let Some(item_str) = item.as_string() {
                let _ = required_attributes.insert(item_str.to_string());
            } else {
                return Err(SchemaError::Malformed {
                    path: schema.path.join(&format!("required/{}", index)).unwrap(),
                    keyword: KeywordType::Required,
                    detail: format!("Required values must be of type {} (found type {})", PrimitiveType::String, item.primitive_type()),
                });
            }
        }

        required_attributes.shrink_to_fit();
        Ok(Some(Self { required_attributes }))
    }

    fn keyword_type(&self) -> KeywordType {
        KeywordType::Required
    }

    fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        if let Some(object) = value.as_object() {
            let keys_iterator = object.keys();
            let matched_attributes = keys_iterator.filter(|key| self.required_attributes.contains(*key)).collect::<HashSet<_>>();

            if matched_attributes.len() == self.required_attributes.len() {
                ValidationErrorIterator::new(EmptyIterator::new())
            } else {
                let missing_attributes = self
                    .required_attributes
                    .iter()
                    .filter_map(|attribute| {
                        if matched_attributes.contains(attribute as &str) {
                            None
                        } else {
                            Some(attribute.as_ref())
                        }
                    })
                    .collect::<Vec<&str>>();
                ValidationErrorIterator::from(ValidationError::new(
                    path,
                    KeywordType::Required,
                    format!("Missing attributes: {}", missing_attributes.join(",")),
                ))
            }
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
    use super::Required;
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
    fn no_required_schema() -> Schema {
        draft4_schema(rust_type!({"required": []})).expect("Schema is supposed to be valid")
    }

    fn prop_required_schema() -> Schema {
        draft4_schema(rust_type!({"required": ["prop"]})).expect("Schema is supposed to be valid")
    }

    // Tests
    #[test_case(
        rust_type![{"required": []}],
        &hash_set!{}
    )]
    #[test_case(
        rust_type![{"required": ["property"]}],
        &hash_set!{"property".to_string()}
    )]
    fn build_required_object_valid(raw_schema: RustType, expected_required_attributes: &HashSet<String>) {
        let schema = draft4_schema(raw_schema).expect("Schema is supposed to be valid");
        schema.do_on_validator(KeywordType::Required, &|maybe_draft_validator| {
            let draft_validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert!(matches!(
                draft_validator,
                DraftValidator::Required(Required { required_attributes, ..})
                if required_attributes == expected_required_attributes
            ))
        });
    }

    #[test_case(
        rust_type!({"required": 1}),
        &format!("Required attribute must be of type {} (found type {})", PrimitiveType::Array, PrimitiveType::Integer)
    )]
    #[test_case(
        rust_type!({"required": [2]}),
        &format!("Required values must be of type {} (found type {})", PrimitiveType::String, PrimitiveType::Integer)
    )]
    fn build_ref_object_invalid(raw_schema: RustType, expected_malformed_error_detail: &str) {
        assert!(matches!(
            draft4_schema(raw_schema),
            Err(SchemaError::Malformed {
                keyword: KeywordType::Required,
                detail,
                ..
            }) if detail == expected_malformed_error_detail
        ));
    }

    #[test_case(&no_required_schema(), &rust_type!("text"), &[])]
    #[test_case(&no_required_schema(), &rust_type!({}), &[])]
    #[test_case(&no_required_schema(), &rust_type!({"prop": {}}), &[])]
    #[test_case(&prop_required_schema(), &rust_type!("text"), &[])]
    #[test_case(&prop_required_schema(), &rust_type!({}), &[ValidationError::new("#", KeywordType::Required, "Missing attributes: prop")])]
    #[test_case(&prop_required_schema(), &rust_type!({"prop": {}}), &[])]
    fn validate(schema: &Schema, object: &RustType, expected_validation_errors: &[ValidationError]) {
        schema.do_on_validator(KeywordType::Required, &|maybe_draft_validator| {
            let validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert_eq!(validator.validation_errors("#", object).collect::<Vec<_>>(), expected_validation_errors);
        });
    }
}

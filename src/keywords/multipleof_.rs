use crate::{
    iterator_utils::empty_iterator::EmptyIterator,
    types::{
        keyword_type::KeywordType, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validation_error::ValidationError, validator::Validator,
        validator_error_iterator::ValidationErrorIterator,
    },
};
use json_trait_rs::{JsonType, PrimitiveType};
use std::any::Any;

#[derive(Debug, Clone, Copy)]
pub(in crate) enum Number {
    Integer(i128),
    Float(f64),
}

#[cfg(test)]
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Integer(value) => {
                if let Self::Integer(other_value) = other {
                    value == other_value
                } else {
                    false
                }
            }
            Self::Float(value) => {
                if let Self::Float(other_value) = other {
                    (value - other_value).abs() <= 2_f64 * f64::EPSILON
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate) struct MultipleOf {
    pub(in crate) value: Number,
}

#[allow(unsafe_code)]
unsafe impl Sync for MultipleOf {}
#[allow(unsafe_code)]
unsafe impl Send for MultipleOf {}

macro_rules! validation_error_message {
    ($dividend:ident, $divisor:ident) => {
        format!("{} is not a multiple of {}", $dividend, $divisor)
    };
}

fn validate_exact_division_i128(path: &str, dividend: i128, divisor: i128) -> ValidationErrorIterator {
    if dividend % divisor == 0 {
        ValidationErrorIterator::new(EmptyIterator::new())
    } else {
        ValidationErrorIterator::from(ValidationError::new(path, KeywordType::MultipleOf, validation_error_message!(dividend, divisor)))
    }
}

fn validate_exact_division_f64(path: &str, dividend: f64, divisor: f64) -> ValidationErrorIterator {
    if dividend % divisor <= 2_f64 * f64::EPSILON {
        ValidationErrorIterator::new(EmptyIterator::new())
    } else {
        ValidationErrorIterator::from(ValidationError::new(path, KeywordType::MultipleOf, validation_error_message!(dividend, divisor)))
    }
}

#[allow(clippy::cast_precision_loss)]
const fn i128_to_f64(value: i128) -> f64 {
    value as f64
}

impl Validator for MultipleOf {
    fn compile<T: 'static + JsonType>(_scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Option<Self>, SchemaError> {
        let multiple_of_attribute = if let Some(value) = schema.get_attribute("multipleOf") {
            value
        } else {
            // multipleOf attribute is not there so we're done here
            return Ok(None);
        };

        let number = if let Some(number) = multiple_of_attribute.as_integer() {
            if number <= 0 {
                return Err(SchemaError::Malformed {
                    path: schema.path.clone(),
                    keyword: KeywordType::MultipleOf,
                    detail: format!("multipleOf value must be strictly greather than 0 (found {})", number),
                });
            }
            Number::Integer(number)
        } else if let Some(number) = multiple_of_attribute.as_number() {
            if number <= 0.0 {
                return Err(SchemaError::Malformed {
                    path: schema.path.clone(),
                    keyword: KeywordType::MultipleOf,
                    detail: format!("multipleOf value must be strictly greather than 0 (found {})", number),
                });
            }
            Number::Float(number)
        } else {
            return Err(SchemaError::Malformed {
                path: schema.path.clone(),
                keyword: KeywordType::MultipleOf,
                detail: format!(
                    "multipleOf attribute must be of type {} or {} (found type {})",
                    PrimitiveType::Integer,
                    PrimitiveType::Number,
                    multiple_of_attribute.primitive_type(),
                ),
            });
        };
        Ok(Some(Self { value: number }))
    }

    fn keyword_type(&self) -> KeywordType {
        KeywordType::MultipleOf
    }

    fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        if let Some(value_number) = value.as_integer() {
            match self.value {
                Number::Integer(multiply_of) => validate_exact_division_i128(path, value_number, multiply_of),
                Number::Float(multiply_of) => validate_exact_division_f64(path, i128_to_f64(value_number), multiply_of),
            }
        } else if let Some(value_number) = value.as_number() {
            match self.value {
                Number::Float(multiply_of) => validate_exact_division_f64(path, value_number, multiply_of),
                Number::Integer(multiply_of) => validate_exact_division_f64(path, value_number, i128_to_f64(multiply_of)),
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
    use super::{MultipleOf, Number};
    use crate::{
        keywords::DraftValidator,
        types::{
            keyword_type::KeywordType,
            schema::{draft4_schema, Schema},
            schema_error::SchemaError,
            validation_error::ValidationError,
        },
    };
    use json_trait_rs::{rust_type, RustType};
    use test_case::test_case;

    // Testing constants
    fn multiple_of_with_integer() -> Schema {
        draft4_schema(rust_type!({"multipleOf": 2})).expect("Schema is supposed to be valid")
    }

    fn multiple_of_with_float() -> Schema {
        draft4_schema(rust_type!({"multipleOf": 0.6})).expect("Schema is supposed to be valid")
    }

    // Tests
    #[test_case(
        rust_type![{"multipleOf": 1}],
        &Number::Integer(1)
    )]
    #[test_case(
        rust_type![{"multipleOf": 1.2}],
        &Number::Float(1.2)
    )]
    fn build_required_object_valid(raw_schema: RustType, expected_value: &Number) {
        let schema = draft4_schema(raw_schema).expect("Schema is supposed to be valid");
        schema.do_on_validator(KeywordType::MultipleOf, &|maybe_draft_validator| {
            let draft_validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert!(matches!(
                draft_validator,
                DraftValidator::MultipleOf(MultipleOf { value, ..})
                if value == expected_value
            ))
        });
    }

    #[test_case(
        rust_type![{"multipleOf": 0}],
        "multipleOf value must be strictly greather than 0 (found 0)"
    )]
    #[test_case(
        rust_type![{"multipleOf": -1}],
        "multipleOf value must be strictly greather than 0 (found -1)"
    )]
    fn build_ref_object_invalid(raw_schema: RustType, expected_malformed_error_detail: &str) {
        assert!(matches!(
            dbg![draft4_schema(raw_schema)],
            Err(SchemaError::Malformed {
                keyword: KeywordType::MultipleOf,
                detail,
                ..
            }) if detail == expected_malformed_error_detail
        ));
    }

    #[test_case(&multiple_of_with_integer(), &rust_type!(null), &[])]
    #[test_case(&multiple_of_with_integer(), &rust_type!(2), &[])]
    #[test_case(&multiple_of_with_integer(), &rust_type!(3), &[ValidationError::new("#", KeywordType::MultipleOf, "3 is not a multiple of 2")])]
    #[test_case(&multiple_of_with_integer(), &rust_type!(12.0), &[])]
    #[test_case(&multiple_of_with_integer(), &rust_type!(23.4), &[ValidationError::new("#", KeywordType::MultipleOf, "23.4 is not a multiple of 2")])]
    #[test_case(&multiple_of_with_float(), &rust_type!(null), &[])]
    #[test_case(&multiple_of_with_float(), &rust_type!(2), &[ValidationError::new("#", KeywordType::MultipleOf, "2 is not a multiple of 0.6")])]
    #[test_case(&multiple_of_with_float(), &rust_type!(3), &[])]
    #[test_case(&multiple_of_with_float(), &rust_type!(12.0), &[])]
    #[test_case(&multiple_of_with_float(), &rust_type!(23.4), &[ValidationError::new("#", KeywordType::MultipleOf, "23.4 is not a multiple of 0.6")])]
    fn validate(schema: &Schema, object: &RustType, expected_validation_errors: &[ValidationError]) {
        schema.do_on_validator(KeywordType::MultipleOf, &|maybe_draft_validator| {
            let validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert_eq!(validator.validation_errors("#", object).collect::<Vec<_>>(), expected_validation_errors);
        });
    }
}

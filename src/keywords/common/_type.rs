maybe_import_dependencies_for_parallel_run!();

use crate::keywords::KeywordAttribute;
use crate::keywords::KeywordKind;
use crate::keywords::KeywordTrait;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::types::EnumPrimitiveType;
use crate::types::PrimitiveType;
use named_type::NamedType;
use named_type_derive::*;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
enum InnerType {
    Single(EnumPrimitiveType),
    Multiple(Vec<EnumPrimitiveType>),
}

impl InnerType {
    #[inline]
    fn is_valid(&self, enum_primitive_value: EnumPrimitiveType) -> bool {
        match self {
            InnerType::Single(value) => *value == enum_primitive_value || (enum_primitive_value == EnumPrimitiveType::Integer && *value == EnumPrimitiveType::Number),
            InnerType::Multiple(values) => {
                for value in values {
                    if *value == enum_primitive_value || (enum_primitive_value == EnumPrimitiveType::Integer && *value == EnumPrimitiveType::Number) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

#[derive(Debug, PartialEq, NamedType)]
pub struct Type<T>
where
    T: PrimitiveType<T>,
{
    inner: InnerType,
    path: Url,
    raw_schema: Box<T>,
}

impl<T> Type<T>
where
    T: PrimitiveType<T>,
{
    #[inline]
    fn _is_valid_type(type_primitive: &T) -> bool {
        type_primitive.as_string().and_then(EnumPrimitiveType::from_type).is_some()
    }
}

impl<T> KeywordAttribute for Type<T>
where
    T: PrimitiveType<T>,
{
    const ATTRIBUTE: &'static str = "type";
}

impl<T> KeywordTrait<T> for Type<T>
where
    T: 'static + PrimitiveType<T>,
{
    fn kind(&self) -> KeywordKind {
        KeywordKind::Type
    }

    fn create_from_valid_schema<L>(_scoped_schema: &mut ScopedSchema<T, L>, path: Url, raw_schema: Box<T>) -> Self
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        let cloned_raw_schema = raw_schema.clone();
        let type_primitive_value = Self::_condition_already_verified(cloned_raw_schema.get(Self::ATTRIBUTE));
        match type_primitive_value.primitive_type() {
            Some(EnumPrimitiveType::String) => Self {
                inner: InnerType::Single(Self::_condition_already_verified(EnumPrimitiveType::from_type(Self::_condition_already_verified(
                    type_primitive_value.as_string(),
                )))),
                path,
                raw_schema,
            },
            Some(EnumPrimitiveType::Array) => Self {
                inner: InnerType::Multiple(
                    iterate![Self::_condition_already_verified(type_primitive_value.as_array())]
                        .map(|value| Self::_condition_already_verified(EnumPrimitiveType::from_type(Self::_condition_already_verified(value.as_string()))))
                        .collect(),
                ),
                path,
                raw_schema,
            },
            _ => Self::_condition_already_verified(None), // Getting to this point triggers a panic, the condition is already been validated
        }
    }

    fn is_keyword(raw_schema: &T) -> bool
    where
        Self: Sized,
    {
        raw_schema.has_attribute(Self::ATTRIBUTE)
    }

    fn schema_validation_error<L>(_scoped_schema: &mut ScopedSchema<T, L>, path: &Url, raw_schema: &T) -> Option<ValidationError<T>>
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        if let Some(type_attribute) = raw_schema.get(Self::ATTRIBUTE) {
            match type_attribute.primitive_type() {
                Some(EnumPrimitiveType::String) => {
                    return if Self::_is_valid_type(type_attribute) {
                        None
                    } else {
                        Some(ValidationError::new(KeywordKind::Type, path))
                    };
                }
                Some(EnumPrimitiveType::Array) => {
                    if type_attribute.as_array().unwrap().iter().any(|&type_primitive| !Self::_is_valid_type(type_primitive)) {
                        Some(ValidationError::new(KeywordKind::Type, path))
                    } else {
                        None
                    }
                }
                _ => {
                    return Some(ValidationError::new(KeywordKind::Type, path));
                }
            }
        } else {
            None
        }
    }

    fn validation_error(&self, value: &T) -> Option<ValidationError<T>> {
        match value.primitive_type() {
            None => None,
            Some(primitive_type) => {
                if self.inner.is_valid(primitive_type) {
                    None
                } else {
                    Some(ValidationError::new(KeywordKind::Type, &self.path))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Type;
    use crate::testing_prelude::*;
    use test_case_derive::test_case;
    use url::Url;

    lazy_static! {
        static ref NO_TYPE_KEYWORD: TestingType = testing_map![];
        static ref TYPE_INTEGER_MAP: TestingType = testing_map![
            "type" => "integer",
        ];
        static ref TYPES_INTEGER_STRING_MAP: TestingType = testing_map![
            "type" => testing_vec!["integer", "string"],
        ];
        static ref TYPE_INT_MAP: TestingType = testing_map![
            "type" => "int",
        ];
        static ref TYPE_INT_STRING_MAP: TestingType = testing_map![
            "type" => testing_vec!["int", "string"],
        ];
    }

    #[test]
    fn test_type() {
        let raw_schema: TestingType = TYPE_INTEGER_MAP.clone();
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        assert_eq!(Type::new(&mut scoped_schema, path, Box::new(raw_schema)).ok().unwrap().kind(), KeywordKind::Type);
    }

    #[test_case(NO_TYPE_KEYWORD.clone(), true, false)]
    #[test_case(TYPE_INTEGER_MAP.clone(), true, true)]
    #[test_case(TYPES_INTEGER_STRING_MAP.clone(), true, true)]
    #[test_case(TYPE_INT_MAP.clone(), false, true)]
    #[test_case(TYPE_INT_STRING_MAP.clone(), false, false)]
    fn test_new(raw_schema: TestingType, is_valid: bool, is_keyword_built: bool) {
        let type_keyword_optional = create_scoped_schema_from_raw_schema(&raw_schema, true)
            .and_then(|scoped_schema: ScopedSchema<TestingType, TestingLoader>| Ok(scoped_schema.keyword(KeywordKind::Type)))
            .ok();

        assert_eq!(type_keyword_optional.is_some(), is_valid);

        if is_valid {
            assert_eq!(type_keyword_optional.unwrap().is_some(), is_keyword_built);
        }
    }

    #[test]
    fn test_new_should_panic_if_validate_is_false() {
        should_panic!({
            let _: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&TYPE_INT_STRING_MAP.clone(), false).ok().unwrap();
        });
    }

    #[test_case(testing_map!["type" => ()], true)]
    #[test_case(testing_map![], false)]
    fn test_is_keyword(raw_schema: TestingType, is_keyword: bool) {
        assert_eq!(Type::is_keyword(&raw_schema), is_keyword);
    }

    #[test_case(TestingType::from(""), None)]
    #[test_case(testing_map!["properties" => testing_map![],], None)]
    #[test_case(TYPE_INTEGER_MAP.clone(), None)]
    #[test_case(TYPES_INTEGER_STRING_MAP.clone(), None)]
    #[test_case(TYPE_INT_MAP.clone(), Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/").unwrap())))]
    #[test_case(TYPE_INT_STRING_MAP.clone(), Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/").unwrap())))]
    fn test_schema_validation_error(raw_schema: TestingType, expected_optional_validation_error: Option<ValidationError<TestingType>>) {
        let optional_validation_error = if let Err(error) = create_scoped_schema_from_raw_schema::<TestingType, TestingLoader>(&raw_schema, true) {
            normalize_urls_in_validation_errors(error, "HOST")
        } else {
            None
        };
        assert_eq!(optional_validation_error, expected_optional_validation_error);
    }

    #[test_case(
        TYPE_INTEGER_MAP.clone(),
        TestingType::from(1),
        None,
    )]
    #[test_case(
        TYPES_INTEGER_STRING_MAP.clone(),
        TestingType::from(1),
        None,
    )]
    #[test_case(
        TYPES_INTEGER_STRING_MAP.clone(),
        TestingType::from(()),
        Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/").unwrap())),
    )]
    #[test_case(
        TYPE_INTEGER_MAP.clone(),
        TestingType::from("2"),
        Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/").unwrap())),
    )]
    #[test_case(
        TYPE_INTEGER_MAP.clone(),
        TestingType::from(false),
        Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/").unwrap())),
    )]
    #[test_case(
        TYPES_INTEGER_STRING_MAP.clone(),
        TestingType::from(false),
        Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/").unwrap())),
    )]
    fn test_validation_error(raw_schema: TestingType, value: TestingType, expected_optional_validation_error: Option<ValidationError<TestingType>>) {
        let scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let type_keyword = scoped_schema.keyword(KeywordKind::Type).unwrap();
        assert_eq!(
            normalize_urls_in_validation_errors(type_keyword.validation_error(&value), "HOST"),
            expected_optional_validation_error
        );
    }
}

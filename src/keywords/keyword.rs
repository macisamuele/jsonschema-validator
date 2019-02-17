// This module will contain a all the jsonschema keywords
// NOTE: to reduce code duplication all the keywords will be defined on a single namespace
// and then collected into "container" draft specific. A lot of keywords have the same definition
// on multiple drafts

maybe_import_dependencies_for_parallel_run!();

use crate::keywords::KeywordKind;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::type_to_str;
use crate::types::PrimitiveType;
use named_type::NamedType;
use std::fmt::Debug;
use url::Url;

pub trait Attribute
where
    Self: Sized,
{
    const ATTRIBUTE: &'static str;
}

pub trait Trait<T>: Debug + Sync + Send + NamedType
where
    T: 'static + PrimitiveType<T>,
{
    // Attributes
    fn kind(&self) -> KeywordKind;

    // Creation
    fn new<L>(scoped_schema: &mut ScopedSchema<T, L>, path: Url, raw_schema: Box<T>) -> Result<Self, Option<ValidationError<T>>>
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        if scoped_schema.config().validate_schema {
            if let Some(validation_error) = Self::schema_validation_error(scoped_schema, &path, &raw_schema) {
                return Err(Some(validation_error));
            }
        }
        Ok(Self::create_from_valid_schema(scoped_schema, path, raw_schema))
    }

    fn create_from_valid_schema<L>(scoped_schema: &mut ScopedSchema<T, L>, path: Url, raw_schema: Box<T>) -> Self
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>;

    // Validation
    fn is_keyword(raw_schema: &T) -> bool
    where
        Self: Sized,
    {
        eprintln!("{}::is_keyword is not implemented yet. [raw_schema: {:?}]", Self::type_name(), raw_schema,);
        true
    }

    fn schema_validation_error<L>(scoped_schema: &mut ScopedSchema<T, L>, path: &Url, raw_schema: &T) -> Option<ValidationError<T>>
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        eprintln!(
            "{}::schema_validation_errors is not implemented yet. [scoped_schema: {:?}, path: {:?}, raw_schema: {:?}]",
            Self::type_name(),
            scoped_schema,
            path,
            raw_schema,
        );
        None
    }

    fn is_schema_valid<L>(scoped_schema: &mut ScopedSchema<T, L>, path: &Url, raw_schema: &T) -> bool
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        Self::schema_validation_error(scoped_schema, path, raw_schema).is_none()
    }

    fn validation_error(&self, value: &T) -> Option<ValidationError<T>> {
        eprintln!("{}::validation_error not implemented yet [self: {:?}, value: {:?}]", type_to_str::<T>(), self, value);
        None
    }

    fn is_valid(&self, value: &T) -> bool {
        self.validation_error(value).is_none()
    }

    // Miscellaneous
    #[inline]
    fn _condition_already_verified<AT>(value: Option<AT>) -> AT
    where
        Self: Sized,
    {
        value.expect("Apparently the schema was not valid and you disabled validation. Please make sure to check the used schema and in general keep validation enabled for schema retrieved from untrusted/remote sources")
    }
}

#[cfg(test)]
mod tests_trait_defaults_methods {
    use super::Trait;
    use crate::testing_prelude::*;
    use url::Url;

    #[derive(Clone, Debug, PartialEq, NamedType)]
    struct FakeKeyword;

    impl Trait<TestingType> for FakeKeyword {
        fn kind(&self) -> KeywordKind {
            KeywordKind::Type
        }

        fn create_from_valid_schema<L>(_scoped_schema: &mut ScopedSchema<TestingType, L>, _path: Url, _raw_schema: Box<TestingType>) -> Self
        where
            Self: Sized,
            L: Loader<TestingType>,
            LoaderError<L::FormatError>: From<L::FormatError>,
        {
            FakeKeyword
        }
    }

    #[test]
    fn test_new() {
        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        assert_eq!(FakeKeyword::new(&mut scoped_schema, path, Box::new(raw_schema)).ok().unwrap(), FakeKeyword);
    }

    #[test]
    fn test_default_is_keyword() {
        assert_eq!(FakeKeyword::is_keyword(&TestingType::from(())), true);
    }

    #[test]
    fn test_default_schema_validation_error() {
        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        assert_eq!(FakeKeyword::schema_validation_error(&mut scoped_schema, &path, &raw_schema), None);
    }

    #[test]
    fn test_is_schema_valid() {
        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        assert_eq!(FakeKeyword::is_schema_valid(&mut scoped_schema, &path, &raw_schema), true);
    }

    #[test]
    fn test_default_validation_errors() {
        fn trait_call<K: Trait<TestingType>>(keyword: &K, value: &TestingType) {
            assert_eq!(keyword.validation_error(value), None);
        }

        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        let keyword = FakeKeyword::new(&mut scoped_schema, path, Box::new(raw_schema)).ok().unwrap();

        trait_call(&keyword, &TestingType::from(()));
    }

    #[test]
    fn test_is_valid() {
        fn trait_call<K: Trait<TestingType>>(keyword: &K, value: &TestingType) {
            assert_eq!(keyword.is_valid(value), true);
        }

        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        let keyword = FakeKeyword::new(&mut scoped_schema, path, Box::new(raw_schema)).ok().unwrap();

        trait_call(&keyword, &TestingType::from(()));
    }
}

#[cfg(test)]
mod tests_keyword_trait {
    use super::Trait;
    use crate::testing_prelude::*;
    use url::Url;

    #[derive(Clone, Debug, PartialEq, NamedType)]
    struct FakeKeyword {
        inner: String,
    }

    impl Trait<TestingType> for FakeKeyword {
        fn kind(&self) -> KeywordKind {
            KeywordKind::Type
        }

        fn create_from_valid_schema<L>(scoped_schema: &mut ScopedSchema<TestingType, L>, _path: Url, _raw_schema: Box<TestingType>) -> Self
        where
            Self: Sized,
            L: Loader<TestingType>,
            LoaderError<L::FormatError>: From<L::FormatError>,
        {
            Self {
                inner: get_path_from_scoped_schema(scoped_schema).to_string(),
            }
        }

        fn schema_validation_error<L>(scoped_schema: &mut ScopedSchema<TestingType, L>, _path: &Url, raw_schema: &TestingType) -> Option<ValidationError<TestingType>>
        where
            L: Loader<TestingType>,
            LoaderError<L::FormatError>: From<L::FormatError>,
        {
            if raw_schema.is_string() {
                None
            } else {
                Some(ValidationError::new(KeywordKind::Type, &get_path_from_scoped_schema(scoped_schema)))
            }
        }

        fn validation_error(&self, value: &TestingType) -> Option<ValidationError<TestingType>> {
            if value.is_number() {
                None
            } else {
                Some(ValidationError::new(KeywordKind::Type, &Url::parse(&self.inner).unwrap()))
            }
        }
    }

    #[test]
    fn test_condition_already_verified_success() {
        assert_eq!(FakeKeyword::_condition_already_verified(Option::from(1)), 1);
    }

    #[test]
    fn test_condition_already_verified_fail() {
        should_panic!({
            FakeKeyword::_condition_already_verified({
                let thing: Option<i32> = None;
                thing
            })
        });
    }

    #[test]
    fn test_new() {
        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);

        assert_eq!(
            FakeKeyword::new(&mut scoped_schema, path.clone(), Box::new(raw_schema)).unwrap_err(),
            Some(ValidationError::new(KeywordKind::Type, &path))
        );

        let raw_schema = TestingType::from("");
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);

        assert_eq!(
            FakeKeyword::new(&mut scoped_schema, path.clone(), Box::new(raw_schema)).ok().unwrap(),
            FakeKeyword { inner: path.to_string() }
        );
    }

    #[test]
    fn test_schema_validation_errors() {
        let raw_schema = TestingType::from(());
        let mut scoped_schema: ScopedSchema<_, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        assert_eq!(
            FakeKeyword::schema_validation_error(&mut scoped_schema, &path, &raw_schema),
            Some(ValidationError::new(KeywordKind::Type, &path))
        );

        let raw_schema = TestingType::from("");
        let mut scoped_schema: ScopedSchema<_, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        assert_eq!(FakeKeyword::schema_validation_error(&mut scoped_schema, &path, &raw_schema), None);
    }

    #[test]
    fn test_is_valid() {
        let raw_schema = TestingType::from("");
        let mut scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let path = get_path_from_scoped_schema(&scoped_schema);
        let keyword = FakeKeyword::new(&mut scoped_schema, path, Box::new(raw_schema)).unwrap();
        assert_eq!(keyword.is_valid(&TestingType::from(1)), true);
        assert_eq!(keyword.is_valid(&TestingType::from("1.2")), false);
    }
}

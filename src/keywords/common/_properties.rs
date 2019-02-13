maybe_import_dependencies_for_parallel_run!();

use crate::keywords::KeywordAttribute;
use crate::keywords::KeywordKind;
use crate::keywords::KeywordTrait;
use crate::loaders::Loader;
use crate::loaders::LoaderError;
use crate::schema::Schema;
use crate::schema::ScopedSchema;
use crate::schema::ValidationError;
use crate::types::EnumPrimitiveType;
use crate::types::JsonMap;
use crate::types::PrimitiveType;
use crate::url_helpers::append_fragment_components;
use named_type::NamedType;
use named_type_derive::*;
use std::ops::Deref;
use url::Url;

#[derive(Debug)]
struct Property<T>
where
    T: 'static + PrimitiveType<T>,
{
    name: String,
    path: Url,
    schema: Schema<T>,
}

impl<T> Deref for Property<T>
where
    T: 'static + PrimitiveType<T>,
{
    type Target = Schema<T>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.schema
    }
}

#[derive(Debug, NamedType)]
pub struct Properties<T>
where
    T: 'static + PrimitiveType<T>,
{
    path: Url,
    properties: Vec<Property<T>>,
}

impl<T> Properties<T>
where
    T: 'static + PrimitiveType<T>,
{
    fn property_schema<L>(scoped_schema: &ScopedSchema<T, L>, path: &Url, raw_schema: Box<T>, property_name: &str) -> Result<Schema<T>, Option<ValidationError<T>>>
    where
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        Schema::new(scoped_schema, &append_fragment_components(path, vec!["properties", property_name]), raw_schema)
    }

    fn new_property_from_valid_schema<L>(scoped_schema: &ScopedSchema<T, L>, path: &Url, raw_schema: Box<T>, property_name: &str) -> Result<Property<T>, Option<ValidationError<T>>>
    where
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        match Self::property_schema(scoped_schema, path, raw_schema, property_name) {
            Ok(schema) => Ok(Property {
                name: property_name.to_string(),
                path: schema.path().clone(),
                schema,
            }),
            Err(validation_errors) => Err(validation_errors),
        }
    }
}

impl<T> KeywordAttribute for Properties<T>
where
    T: PrimitiveType<T>,
{
    const ATTRIBUTE: &'static str = "properties";
}

impl<T> KeywordTrait<T> for Properties<T>
where
    T: 'static + PrimitiveType<T>,
{
    fn kind(&self) -> KeywordKind {
        KeywordKind::Properties
    }

    fn create_from_valid_schema<L>(scoped_schema: &ScopedSchema<T, L>, path: Url, raw_schema: Box<T>) -> Self
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        let properties_map: JsonMap<T> = Self::_condition_already_verified(Self::_condition_already_verified(raw_schema.get(Self::ATTRIBUTE)).as_object());

        Self {
            properties: iterate![properties_map.items().unwrap_or_default()]
                .map(|(property_name, property_raw_schema)| {
                    Self::_condition_already_verified(
                        Self::new_property_from_valid_schema(
                            scoped_schema,
                            &path,
                            Box::new({
                                let thing: &T = property_raw_schema;
                                thing.clone()
                            }),
                            property_name,
                        )
                        .ok(),
                    )
                })
                .collect(),
            path,
        }
    }

    fn is_keyword(raw_schema: &T) -> bool
    where
        Self: Sized,
    {
        raw_schema.has_attribute(Self::ATTRIBUTE)
    }

    fn schema_validation_error<L>(scoped_schema: &ScopedSchema<T, L>, path: &Url, raw_schema: &T) -> Option<ValidationError<T>>
    where
        Self: Sized,
        L: Loader<T>,
        LoaderError<L::FormatError>: From<L::FormatError>,
    {
        if let Some(properties) = raw_schema.get(Self::ATTRIBUTE) {
            // We're guaranteed to have it
            return if properties.primitive_type() == Some(EnumPrimitiveType::Object) {
                iterate![properties.as_object().unwrap().items().unwrap_or_default()]
                    .filter_map(|(property_name, raw_property_schema)| match raw_property_schema.primitive_type() {
                        Some(EnumPrimitiveType::Object) => match Self::property_schema(scoped_schema, path, Box::new((**raw_property_schema).clone()), property_name) {
                            Ok(_) => None,
                            Err(validation_errors) => validation_errors,
                        },
                        _ => Some(ValidationError::<T>::new(KeywordKind::Properties, path)),
                    })
                    .collect::<Vec<ValidationError<T>>>()
                    .pop()
            } else {
                Some(ValidationError::new(KeywordKind::Properties, path))
            };
        } else {
            None
        }
    }

    fn validation_error(&self, value: &T) -> Option<ValidationError<T>> {
        iterate![self.properties]
            .filter_map(|property: &Property<T>| value.get(property.name.as_str()).and_then(|property_value| property.validation_error(property_value)))
            .collect::<Vec<ValidationError<T>>>()
            .pop()
    }
}

#[cfg(test)]
mod tests {
    use super::Properties;
    use crate::testing_prelude::*;
    use test_case_derive::test_case;
    use url::Url;

    lazy_static! {
        static ref NO_PROPERTIES_KEYWORD: TestingType = testing_map![];
        static ref NO_PROPERTIES: TestingType = testing_map![
            "properties" => testing_map![],
        ];
        static ref SINGLE_PROPERTY_STRING_TYPE: TestingType = testing_map![
            "properties" => testing_map![
                "prop1" => testing_map![
                    "type" => "string",
                ],
            ],
        ];
        static ref INVALID_PROPERTIES: TestingType = testing_map![
            "properties" => (),
        ];
    }

    #[test]
    fn test_type() {
        let raw_schema = SINGLE_PROPERTY_STRING_TYPE.clone();
        let scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();

        assert_eq!(
            Properties::new(&scoped_schema, get_path_from_scoped_schema(&scoped_schema), Box::new(raw_schema))
                .ok()
                .unwrap()
                .kind(),
            KeywordKind::Properties
        );
    }

    #[test_case(NO_PROPERTIES_KEYWORD.clone(), true, false)]
    #[test_case(NO_PROPERTIES.clone(), true, true)]
    #[test_case(SINGLE_PROPERTY_STRING_TYPE.clone(), true, true)]
    #[test_case(INVALID_PROPERTIES.clone(), false, false)]
    fn test_new(raw_schema: TestingType, is_valid: bool, is_keyword_built: bool) {
        let properties_keyword_optional = create_scoped_schema_from_raw_schema(&raw_schema, true)
            .and_then(|scoped_schema: ScopedSchema<TestingType, TestingLoader>| Ok(scoped_schema.keyword(KeywordKind::Properties)))
            .ok();

        assert_eq!(properties_keyword_optional.is_some(), is_valid);

        if is_valid {
            assert_eq!(properties_keyword_optional.unwrap().is_some(), is_keyword_built);
        }
    }

    #[test]
    fn test_new_should_panic_if_validate_is_false() {
        should_panic!({
            let _: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&INVALID_PROPERTIES.clone(), false).ok().unwrap();
        });
    }

    #[test_case(testing_map!["properties" => ()], true)]
    #[test_case(testing_map![], false)]
    fn test_is_keyword(raw_schema: TestingType, is_keyword: bool) {
        assert_eq!(Properties::is_keyword(&raw_schema), is_keyword);
    }

    #[test_case(TestingType::from(()), None)]
    #[test_case(testing_map![], None)]
    #[test_case(TestingType::from("string"), None)]
    #[test_case(TestingType::from(1), None)]
    #[test_case(testing_map!["properties" => testing_map![]], None)]
    #[test_case(testing_map!["properties" => testing_map!["property" => testing_map![]]], None)]
    #[test_case(
        testing_map!["properties" => ()],
        Some(ValidationError::new(KeywordKind::Properties, &Url::parse("memory://HOST/#/").unwrap())),
    )]
    #[test_case(
        testing_map!["properties" => testing_map!["property" => ()]],
        Some(ValidationError::new(KeywordKind::Properties, &Url::parse("memory://HOST/#/").unwrap())),
    )]
    #[test_case(
        testing_map!["properties" => testing_map!["property" => testing_map!["type" => "invalid_type"]]],
        Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/properties/property").unwrap())),
    )]
    fn test_schema_validation_error(raw_schema: TestingType, expected_optional_validation_error: Option<ValidationError<TestingType>>) {
        let optional_validation_error = if let Err(error) = create_scoped_schema_from_raw_schema::<TestingType, TestingLoader>(&raw_schema, true) {
            normalize_urls_in_validation_errors(error, "HOST")
        } else {
            None
        };
        assert_eq!(optional_validation_error, expected_optional_validation_error);
    }

    #[test]
    fn test_properties_are_registered() {
        let raw_schema = SINGLE_PROPERTY_STRING_TYPE.clone();
        let scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let properties = Properties::new(&scoped_schema, get_path_from_scoped_schema(&scoped_schema), Box::new(raw_schema))
            .ok()
            .unwrap();

        assert_eq!(properties.properties.len(), 1);
        assert_eq!(properties.properties[0].name, "prop1".to_string());
    }

    #[test_case(
        testing_map!["key1" => "value", "key2" => 1, "key3" => (), "key4" => false],
        None,
    )]
    #[test_case(
        testing_map!["property" => "value"],
        Some(ValidationError::new(KeywordKind::Type, &Url::parse("memory://HOST/#/properties/property").unwrap())),
    )]
    fn test_validation_error(value: TestingType, expected_optional_validation_error: Option<ValidationError<TestingType>>) {
        let raw_schema = testing_map![
            "properties" => testing_map![
                "property" => testing_map![
                    "type" => "object",
                ],
            ],
        ];
        let scoped_schema: ScopedSchema<TestingType, TestingLoader> = create_scoped_schema_from_raw_schema(&raw_schema, true).ok().unwrap();
        let properties_keyword = scoped_schema.keyword(KeywordKind::Properties).unwrap();
        assert_eq!(
            normalize_urls_in_validation_errors(properties_keyword.validation_error(&value), "HOST"),
            expected_optional_validation_error
        );
    }
}

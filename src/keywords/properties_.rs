use crate::{
    iterator_utils::empty_iterator::EmptyIterator,
    types::{
        keyword_type::KeywordType, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validator::Validator, validator_error_iterator::ValidationErrorIterator,
    },
};
use json_trait_rs::{JsonMapTrait, JsonType, PrimitiveType};
use std::{any::Any, collections::HashMap, fmt::Debug, sync::Arc};
use url::Url;

#[derive(Debug)]
pub(in crate) struct Properties {
    pub(in crate) properties: HashMap<String, Arc<Schema>>,
}

#[allow(unsafe_code)]
unsafe impl Sync for Properties {}
#[allow(unsafe_code)]
unsafe impl Send for Properties {}

fn into_str<'l, S: Into<&'l str>>(value: S) -> &'l str {
    value.into()
}

fn malformed_properties<J: JsonType>(path: &Url, value: &J) -> SchemaError {
    SchemaError::Malformed {
        path: path.clone(),
        keyword: KeywordType::Properties,
        detail: format!(
            "Values have to be of `{}` type. {:?} has type `{}`.",
            into_str(PrimitiveType::Object),
            value,
            into_str(value.primitive_type())
        ),
    }
}

fn property_path(schema_path: &Url, property_name: &str) -> Url {
    let mut result = schema_path.clone();
    result.set_fragment(Some(&format!(
        "{}/properties/{}",
        schema_path.fragment().unwrap_or("").trim_end_matches('/'),
        property_name,
    )));
    result
}

impl Validator for Properties {
    fn compile<T: 'static + JsonType>(scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Option<Self>, SchemaError>
    where
        Self: Sized,
    {
        let properties_attribute = if let Some(value) = schema.get_attribute("properties") {
            value
        } else {
            // properties attribute is not there so we're done here
            return Ok(None);
        };
        let properties_map = if let Some(value) = properties_attribute.as_object() {
            value
        } else {
            return Err(malformed_properties(&schema.path, properties_attribute));
        };

        let (properties, errors) = properties_map
            .items()
            .map(|(key, value)| {
                let path = property_path(&schema.path, key);
                if value.is_object() {
                    match scope_builder.schema(&path, value) {
                        Ok(arc_schema) => Ok((key.to_string(), arc_schema)),
                        Err(schema_error) => {
                            unreachable!(
                                "Unexpected error in the schema: path: {:?}, raw_schema: {:?}, schema_error: {:?}",
                                path, value, schema_error,
                            );
                        }
                    }
                } else {
                    Err(path)
                }
            })
            .partition::<Vec<_>, _>(Result::is_ok);

        if errors.is_empty() {
            Ok(Some(Self {
                properties: properties
                    .into_iter()
                    .map(|result| match result {
                        Ok((path, arc_schema)) => (path, arc_schema),
                        Err(path) => unreachable!("No errors are expected in the properties. path: {:?}", path),
                    })
                    .collect::<HashMap<_, _>>(),
            }))
        } else {
            let faulty_urls = errors
                .iter()
                .map(|result| match result {
                    Ok(arc_schema) => unreachable!("No successes are expected in the errors. schema: {:?}", arc_schema),
                    Err(path) => path.as_str(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            Err(SchemaError::Malformed {
                path: schema.path.clone(),
                keyword: KeywordType::Properties,
                detail: format!(
                    "Values of properties object must be of object type. The following URLs are recognized as faulty: {}",
                    faulty_urls
                ),
            })
        }
    }

    fn keyword_type(&self) -> KeywordType {
        KeywordType::Properties
    }

    fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        if let Some(object) = value.as_object() {
            ValidationErrorIterator::new(
                self.properties
                    .iter()
                    .filter_map(|(attribute_name, schema)| {
                        object
                            .get_attribute(attribute_name)
                            .map(|attribute_value| schema.validation_errors(&format!("{}/{}", path, attribute_name), attribute_value))
                    })
                    .flatten()
                    .collect::<Vec<_>>(),
            )
        } else {
            ValidationErrorIterator::new(EmptyIterator::new())
        }
    }

    fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.properties
            .iter()
            .all(|(attribute_name, schema)| schema.is_valid(&format!("{}/{}", path, attribute_name), value))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::property_path;
    use test_case::test_case;
    use url::Url;

    #[test_case("memory://", "prop", "memory://#/properties/prop")]
    #[test_case("memory:///path", "prop", "memory:///path#/properties/prop")]
    #[test_case("memory:///path#/fragment", "prop", "memory:///path#/fragment/properties/prop")]
    fn test_property_path(schema_url: &str, property_name: &str, expected_url: &str) {
        assert_eq!(property_path(&Url::parse(schema_url).unwrap(), property_name).as_str(), expected_url);
    }
}

#[cfg(test)]
mod validator_tests {
    use super::Properties;
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
    use json_trait_rs::{rust_type, RustType};
    use std::collections::HashSet;
    use test_case::test_case;

    // Testing constants
    fn bool_raw_schema() -> RustType {
        rust_type!({"properties": {"bool": {"type": "boolean"}}})
    }

    fn bool_schema() -> Schema {
        draft4_schema(bool_raw_schema()).expect("Schema is supposed to be valid")
    }

    // Tests
    #[test_case(
        rust_type!({"properties": {"prop": {"type": "integer"}}}),
        &hash_set!["prop".to_string()]
    )]
    #[test_case(
        rust_type!({"properties": {"prop1": {"type": "integer"}, "prop2": {}}}),
        &hash_set!["prop1".to_string(), "prop2".to_string()]
    )]
    fn build_properties_object_valid(raw_schema: RustType, expected_properties: &HashSet<String>) {
        let schema = draft4_schema(raw_schema).expect("Schema is supposed to be valid");
        schema.do_on_validator(KeywordType::Properties, &|maybe_draft_validator| {
            let draft_validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert!(matches!(
                draft_validator,
                DraftValidator::Properties(Properties {properties, ..})
                if &properties.keys().cloned().collect::<HashSet<_>>() == expected_properties
            ))
        });
    }

    #[test_case(
        rust_type!({"properties": 1}),
        "Values have to be of `object` type. Integer(1) has type `integer`."
    )]
    #[test_case(
        rust_type!({"properties": {"prop": 1}}),
        "Values of properties object must be of object type. The following URLs are recognized as faulty: memory://URL_PLACEHOLDER#/properties/prop"
    )]
    fn build_properties_object_invalid(raw_schema: RustType, expected_malformed_error_detail: &str) {
        assert!(matches!(
            draft4_schema(raw_schema),
            Err(SchemaError::Malformed {
                keyword: KeywordType::Properties,
                detail,
                path
            }) if expected_malformed_error_detail.replace("URL_PLACEHOLDER", path.path()) == detail.as_str()
        ));
    }

    #[test_case(&bool_schema(), &rust_type!("text"), &[])]
    #[test_case(&bool_schema(), &rust_type!({"bool": true}), &[])]
    #[test_case(&bool_schema(), &rust_type!({"bool": "wrong type"}), &[
        ValidationError::new("#/bool", KeywordType::Type, "Invalid Type"),
    ])]
    fn validate(schema: &Schema, object: &RustType, expected_validation_errors: &[ValidationError]) {
        schema.do_on_validator(KeywordType::Properties, &|maybe_draft_validator| {
            let validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert_eq!(validator.validation_errors("#", object).collect::<Vec<_>>(), expected_validation_errors);
        });
    }
}

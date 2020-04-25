use crate::types::{
    keyword_type::KeywordType, schema::Schema, schema_error::SchemaError, scope_builder::ScopeBuilder, validator::Validator, validator_error_iterator::ValidationErrorIterator,
};
use json_trait_rs::JsonType;
use std::{any::Any, fmt::Debug, sync::Arc};
use url::Url;

#[derive(Debug, Clone)]
pub(in crate) struct Ref {
    pub(in crate) referenced_uri: Url,
    pub(in crate) referenced_schema: Arc<Schema>,
}

#[allow(unsafe_code)]
unsafe impl Sync for Ref {}
#[allow(unsafe_code)]
unsafe impl Send for Ref {}

fn full_uri(base_path: &Url, json_reference: &str) -> Url {
    base_path.join(json_reference).unwrap()
}

impl Validator for Ref {
    fn compile<T: 'static + JsonType>(scope_builder: &mut ScopeBuilder<T>, schema: &Schema) -> Result<Option<Self>, SchemaError>
    where
        Self: Sized,
    {
        let ref_attribute = if let Some(value) = schema.get_attribute("$ref") {
            value
        } else {
            // $ref attribute is not there so we're done here
            return Ok(None);
        };

        let ref_value = if let Some(value) = ref_attribute.as_string() {
            value
        } else {
            // $ref attribute is not of type object. So we should not consider it
            return Ok(None);
        };

        let referenced_uri = full_uri(&schema.path, ref_value);
        let referenced_raw_schema: Arc<T> = scope_builder.retrieve_schema(&referenced_uri)?;
        let referenced_schema = scope_builder.schema(&referenced_uri, &referenced_raw_schema.to_rust_type())?;
        Ok(Some(Self {
            referenced_uri,
            referenced_schema,
        }))
    }

    fn keyword_type(&self) -> KeywordType {
        KeywordType::Ref
    }

    fn validation_errors<T: 'static + JsonType>(&self, path: &str, value: &T) -> ValidationErrorIterator {
        self.referenced_schema.validation_errors(path, value)
    }

    fn is_valid<T: 'static + JsonType>(&self, path: &str, value: &T) -> bool {
        self.referenced_schema.is_valid(path, value)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::full_uri;
    use test_case::test_case;
    use url::Url;

    // Fragment update only
    #[test_case("memory://d0", "#" => "memory://d0#")]
    #[test_case("memory://d1#", "#" => "memory://d1#")]
    #[test_case("memory://d2#/old/fragment", "#" => "memory://d2#")]
    #[test_case("memory://d3#", "#/new/fragment" => "memory://d3#/new/fragment")]
    #[test_case("memory://d4#/old/fragment", "#/new/fragment" => "memory://d4#/new/fragment")]
    #[test_case("memory://d5/file#", "#/new/fragment" => "memory://d5/file#/new/fragment")]
    #[test_case("memory://d6/file#/old/fragment", "#/new/fragment" => "memory://d6/file#/new/fragment")]
    // Relative within the same "folder"
    #[test_case("memory:///file", "new_file_on_empty_domain" => "memory:///new_file_on_empty_domain")]
    #[test_case("memory://d7/file", "new_file" => "memory://d7/new_file")]
    #[test_case("memory://d8/file", "new_file#/fragment" => "memory://d8/new_file#/fragment")]
    #[test_case("memory://d9/file#/old/fragment", "new_file#/new/fragment" => "memory://d9/new_file#/new/fragment")]
    // Relative and absolute paths
    #[test_case("memory://d10/p0/p1/p2", "./new_path" => "memory://d10/p0/p1/new_path")]
    #[test_case("memory://d11/p0/p1/p2", "../new_path" => "memory://d11/p0/new_path")]
    #[test_case("memory://d12/p0/p1/p2", "/new_path" => "memory://d12/new_path")]
    // Full url substitution
    #[test_case("memory://d13/file", "memory://new_path" => "memory://new_path")]
    fn test_full_uri(base_path: &str, json_reference: &str) -> String {
        full_uri(&Url::parse(base_path).unwrap(), json_reference).as_str().into()
    }
}

#[cfg(test)]
mod validator_tests {
    use super::Ref;
    use crate::{
        keywords::DraftValidator,
        types::{
            keyword_type::KeywordType,
            schema::{draft4_schema, Schema},
            schema_error::SchemaError,
            validation_error::ValidationError,
        },
    };
    use json_trait_rs::{rust_type, JsonTypeToString, PrimitiveType, RustType};
    use loader_rs::{testing_helpers::MockLoaderRequestBuilder, url_helpers::UrlError, LoaderError};
    use test_case::test_case;
    use url::Url;

    // Testing constants
    fn no_validators_schema() -> Schema {
        draft4_schema(rust_type!({
            "definitions": {"model": {}},
            "$ref": "#/definitions/model",
        }))
        .expect("Schema is supposed to be valid")
    }

    fn string_schema() -> Schema {
        draft4_schema(rust_type!({
            "definitions": {"model": {"type": "string"}},
            "$ref": "#/definitions/model",
        }))
        .expect("Schema is supposed to be valid")
    }

    fn string_schema_with_remote_references() -> Schema {
        MockLoaderRequestBuilder::default()
            .resp_body(rust_type!({"valid": {"$ref": "#/string_type"}, "string_type": {"type": "string"}}).to_string())
            .resp_content_type("application/json")
            .http_path("/remote.json")
            .expected_mock_calls(1)
            .build()
            .unwrap()
            .run_in_mock_context(&|url| draft4_schema(rust_type!({ "$ref": format!("{}#/valid", url) })))
            .expect("Schema is supposed to be valid")
    }

    // Tests
    #[test_case(
        rust_type!({"definitions": {"model": {}}, "$ref": "#/definitions/model"}),
        "memory://URL_PLACEHOLDER#/definitions/model"
    )]
    #[test_case(
        rust_type!({"definitions": {"model": {"type": "integer"}}, "$ref": "#/definitions/model"}),
        "memory://URL_PLACEHOLDER#/definitions/model"
    )]
    fn build_ref_object_valid_local_references(raw_schema: RustType, expected_referenced_uri_str: &str) {
        let schema = draft4_schema(raw_schema).expect("Schema is supposed to be valid");
        schema.do_on_validator(KeywordType::Ref, &|maybe_draft_validator| {
            let draft_validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert!(matches!(
                draft_validator,
                DraftValidator::Ref(Ref { referenced_uri, ..})
                if referenced_uri == &Url::parse(&expected_referenced_uri_str.replace("URL_PLACEHOLDER", schema.path.path())).unwrap()
            ))
        });
    }

    #[test]
    fn build_ref_object_valid_remote_references() {
        let (schema, generated_url) = MockLoaderRequestBuilder::default()
            .resp_body(rust_type!({"valid": {}}).to_string())
            .resp_content_type("application/json")
            .http_path("/remote.json")
            .expected_mock_calls(1)
            .build()
            .unwrap()
            .run_in_mock_context(&|url| {
                (
                    draft4_schema(rust_type!({ "$ref": format!("{}#/valid", url) })).expect("Schema is supposed to be valid"),
                    url.clone(),
                )
            });

        schema.do_on_validator(KeywordType::Ref, &|maybe_draft_validator| {
            let draft_validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert!(matches!(
                draft_validator,
                DraftValidator::Ref(Ref { referenced_uri, ..})
                if referenced_uri == &generated_url.join("#/valid").unwrap()

            ))
        });
    }

    #[test_case(
        rust_type!({"definitions": {"model": 1}, "$ref": "#/definitions/model"}),
        &format!("raw_schema type is {} while is expected type {}. raw_schema: {}", PrimitiveType::Integer, PrimitiveType::Object, rust_type!(1))
    )]
    fn build_ref_object_invalid(raw_schema: RustType, expected_malformed_error_detail: &str) {
        assert!(matches!(
            draft4_schema(raw_schema),
            Err(SchemaError::Malformed {
                keyword: KeywordType::Unknown,
                detail,
                ..
            }) if detail == expected_malformed_error_detail
        ));
    }

    #[test_case(
        rust_type!({"$ref": "#/definitions/model"}),
        &format!("Fragment '/definitions/model' not found in {}", rust_type!({"$ref": "#/definitions/model"}).to_json_string())
    )]
    fn build_ref_object_reference_not_found(raw_schema: RustType, expected_json_fragment_error: &str) {
        assert!(matches!(
            draft4_schema(raw_schema),
            Err(SchemaError::LoaderError(LoaderError::InvalidURL(
                UrlError::JsonFragmentError(
                    message
            )))) if message == expected_json_fragment_error
        ));
    }

    #[test_case(&no_validators_schema(), &rust_type!("text"), &[])]
    #[test_case(&no_validators_schema(), &rust_type!(null), &[])]
    #[test_case(&string_schema(), &rust_type!("text"), &[])]
    #[test_case(&string_schema(), &rust_type!(null), &[ValidationError::new("#", KeywordType::Type, "Invalid Type")])]
    #[test_case(&string_schema_with_remote_references(), &rust_type!("text"), &[])]
    #[test_case(&string_schema_with_remote_references(), &rust_type!(null), &[ValidationError::new("#", KeywordType::Type, "Invalid Type")])]
    fn validate(schema: &Schema, object: &RustType, expected_validation_errors: &[ValidationError]) {
        schema.do_on_validator(KeywordType::Ref, &|maybe_draft_validator| {
            let validator = maybe_draft_validator.expect("Expected validator to be found and of correct type");
            assert_eq!(validator.validation_errors("#", object).collect::<Vec<_>>(), expected_validation_errors);
        });
    }
}

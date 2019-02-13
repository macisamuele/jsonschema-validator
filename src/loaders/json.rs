setup_loader![
    serde_json,
    serde_json::Value,
    JSONLoader,
    serde_json::Error,
    |content: String| match serde_json::from_str(&content) {
        Ok(value) => Ok(value),
        Err(serde_error) => Err(serde_error)?,
    },
];

#[cfg(test)]
mod tests {
    use super::JSONLoader;
    use crate::testing_prelude::*;
    use test_case_derive::test_case;

    #[test_case("key_string", true, false, false, json!["value"])]
    #[test_case("key_integer", false, true, false, json![1])]
    #[test_case("key_boolean", false, false, true, json![true])]
    fn test_load_from_string_with_valid_content(key: &str, is_string: bool, is_integer: bool, is_boolean: bool, expected_value: serde_json::Value) {
        let json_object = JSONLoader::load_from_string(String::from(
            r#"
            {
                "key_string": "value",
                "key_integer": 1,
                "key_boolean": true
            }
        "#,
        ))
        .unwrap();

        let extracted_value = json_object.get(key).unwrap();
        assert_eq!(extracted_value.is_string(), is_string);
        assert_eq!(extracted_value.is_integer(), is_integer);
        assert_eq!(extracted_value.is_boolean(), is_boolean);

        assert_eq!(extracted_value, &expected_value);
    }

    #[test]
    fn test_load_from_string_with_invalid_content() {
        let content = r#"
            {
                "key_object": {
            }
        "#;

        expected_err!(JSONLoader::load_from_string(String::from(content)), LoaderError::FormatError, |value| {
            assert_eq!(format!("{:?}", value), "Error(\"EOF while parsing an object\", line: 5, column: 8)",);
        });
    }
}

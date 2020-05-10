use super::{retrieve_test_cases, run_tests};
use std::path::Path;
use test_case::test_case;

#[test_case("type")]
#[test_case("properties")]
// #[test_case("ref")]  // This causes stack overflow. Weird but will be checked later
#[test_case("required")]
fn keyword(file_name_without_extension: &str) {
    run_tests(
        &retrieve_test_cases(Path::new(&format!("JSON-Schema-Test-Suite/tests/draft4/{}.json", file_name_without_extension))),
        &[
            // additionalProperties and patternProperties are not implemented
            "JSON-Schema-Test-Suite/tests/draft4/properties.json#/1/1",
            "JSON-Schema-Test-Suite/tests/draft4/properties.json#/1/2",
            "JSON-Schema-Test-Suite/tests/draft4/properties.json#/1/4",
            "JSON-Schema-Test-Suite/tests/draft4/properties.json#/1/7",
        ],
    )
}

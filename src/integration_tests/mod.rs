// TODO: This should be moved into `tests/` directory once the library is stable

mod draft4;

use crate::types::{draft_version::DraftVersion, scope_builder::ScopeBuilder};
use loader_rs::loaders::SerdeJsonLoader;
use serde_json::Value;
use std::{fs::File, path::Path};

#[derive(Debug, Clone)]
struct TestCase {
    scope_description: String,
    test_description: String,
    schema: Value,
    data: Value,
    valid: bool,
    unique_reference: String,
}

#[derive(Deserialize)]
struct RawTestCase {
    description: String,
    data: Value,
    valid: bool,
}

#[derive(Deserialize)]
struct RawMetadata {
    description: String,
    schema: Value,
    tests: Vec<RawTestCase>,
}

fn parse_test_cases(file_path: &Path) -> Vec<TestCase> {
    let file = File::open(file_path).unwrap();
    let raw_metadata: Vec<RawMetadata> = serde_json::from_reader(file).unwrap();

    let mut test_cases = Vec::new();
    for (metadata_index, metadata) in raw_metadata.iter().enumerate() {
        for (test_index, test) in metadata.tests.iter().enumerate() {
            test_cases.push(TestCase {
                scope_description: metadata.description.clone(),
                test_description: test.description.clone(),
                schema: metadata.schema.clone(),
                data: test.data.clone(),
                valid: test.valid,
                unique_reference: format!("{}#/{}/{}", file_path.display(), metadata_index, test_index),
            });
        }
    }
    test_cases
}

fn retrieve_test_cases(path: &Path) -> Vec<TestCase> {
    if path.is_file() {
        parse_test_cases(path)
    } else {
        let contents = std::fs::read_dir(path).unwrap();

        let mut test_cases = Vec::new();
        for maybe_dir_entry in contents {
            let dir_entry = maybe_dir_entry.unwrap();
            if dir_entry.file_type().unwrap().is_dir() {
                test_cases.extend(retrieve_test_cases(&dir_entry.path()));
            } else {
                test_cases.extend(parse_test_cases(&dir_entry.path()));
            }
        }

        test_cases
    }
}

fn run_test(test_case: &TestCase) -> Result<(), TestCase> {
    let mut scope_builder = ScopeBuilder::create(DraftVersion::Draft4, SerdeJsonLoader::default());
    let schema_url = scope_builder.add_schema(test_case.schema.clone()).unwrap();
    let scope = scope_builder.build(); //.expect("Expected valid schema in the test-cases");

    if scope.is_valid(&schema_url, &test_case.data) == test_case.valid {
        Ok(())
    } else {
        Err(test_case.clone())
    }
}

fn run_tests(test_cases: &[TestCase], ignore_test_failures: &[&str]) {
    for test_case in test_cases {
        if let Err(test_case) = run_test(test_case) {
            if ignore_test_failures.contains(&test_case.unique_reference.as_ref()) {
                eprintln!("[ignored] TestCase has failed. {:?}", test_case);
            } else {
                panic!("At least one test case has failed.\n{:#?}", test_case);
            }
        }
    }
}

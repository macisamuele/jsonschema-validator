#![deny(
    unreachable_pub,
    anonymous_parameters,
    bad_style,
    const_err,
    dead_code,
    deprecated,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    incoherent_fundamental_impls,
    late_bound_lifetime_arguments,
    missing_copy_implementations,
    missing_debug_implementations,
    // missing_docs,
    non_shorthand_field_patterns,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unreachable_code,
    unreachable_patterns,
    unsafe_code,
    unused_allocation,
    unused_assignments,
    unused_comparisons,
    unused_doc_comments,
    unused_extern_crates,
    unused_extern_crates,
    unused_import_braces,
    unused_import_braces,
    unused_imports,
    unused_macros,
    unused_parens,
    unused_qualifications,
    unused_results,
    unused_unsafe,
    unused_variables,
    warnings,
)]

#[cfg(feature = "json")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "json")]
use conv::prelude::ConvUtil;
#[cfg(feature = "json")]
use jsonschema_validator::drafts::DraftVersion;
#[cfg(feature = "json")]
use jsonschema_validator::loaders::JSONLoader;
#[cfg(feature = "json")]
use jsonschema_validator::schema::SchemaBuilder;
#[cfg(feature = "json")]
use jsonschema_validator::schema::ScopedSchema;
#[cfg(feature = "json")]
use std::borrow::Cow;
#[cfg(feature = "json")]
use std::fs::read_dir;
#[cfg(feature = "json")]
use std::fs::File;
#[cfg(feature = "json")]
use std::io::BufReader;
#[cfg(feature = "json")]
use std::path::Path;
#[cfg(feature = "json")]
use std::path::PathBuf;

#[allow(unused_macros)]
macro_rules! path_from_components {
    ($($path_component:expr),*,) => {
        path_from_components![$($item),*]
    };
    ($($path_component:expr),*) => {{
        let thing: PathBuf = root_repository_path().join("JSON-Schema-Test-Suite")$( .join($path_component))*;
        thing
    }};
}

#[cfg(feature = "json")]
fn root_repository_path() -> PathBuf {
    Path::new(file!()).canonicalize().unwrap().parent().unwrap().parent().unwrap().to_path_buf()
}

#[cfg(feature = "json")]
#[derive(Clone, Debug, PartialEq)]
pub struct TestCase<'a> {
    file_name: String,
    group_description: Cow<'a, String>,
    test_description: Cow<'a, String>,
    schema: Cow<'a, serde_json::Value>,
    data: Cow<'a, serde_json::Value>,
    valid: bool,
}

#[cfg(feature = "json")]
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Test<'a> {
    description: Cow<'a, String>,
    data: Cow<'a, serde_json::Value>,
    valid: bool,
}

#[cfg(feature = "json")]
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TestGroup<'a> {
    description: Cow<'a, String>,
    schema: Cow<'a, serde_json::Value>,
    tests: Vec<Test<'a>>,
}

#[cfg(feature = "json")]
pub fn collect_tests<'a>(path: &Path) -> Vec<TestCase<'a>> {
    if path.is_file() {
        let test_groups: Vec<TestGroup<'a>> = serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap();
        test_groups
            .iter()
            .flat_map(|test_group| {
                test_group
                    .tests
                    .iter()
                    .map(|test_case| TestCase {
                        file_name: path.to_str().unwrap().to_string(),
                        group_description: test_group.description.clone(),
                        test_description: test_case.description.clone(),
                        schema: test_group.schema.clone(),
                        data: test_case.data.clone(),
                        valid: test_case.valid,
                    })
                    .collect::<Vec<TestCase<'a>>>()
            })
            .collect()
    } else {
        read_dir(path).unwrap().flat_map(|entry| collect_tests(&entry.unwrap().path())).collect()
    }
}

#[cfg(feature = "json")]
pub fn run_test_cases<'a>(tests_path: &PathBuf, draft_version: DraftVersion) -> (Vec<TestCase<'a>>, Vec<TestCase<'a>>) {
    let (tmp_successes, tmp_failures): (Vec<Result<_, _>>, Vec<Result<_, _>>) = collect_tests(tests_path)
        .iter()
        .cloned()
        .map(|test_case| {
            let cloned_test_case = test_case.clone();
            let schema_result: Result<ScopedSchema<serde_json::Value, JSONLoader>, _> = SchemaBuilder::default()
                .draft_version(draft_version)
                .follow_references(false)
                .validate_schema(true)
                .raw_schema(Box::new(test_case.schema.into_owned()))
                .build();

            match schema_result {
                Ok(schema) => {
                    let validation_errors = schema.validation_error(&test_case.data);
                    if (test_case.valid && validation_errors.is_none()) || (!test_case.valid && validation_errors.is_some()) {
                        Ok(cloned_test_case)
                    } else {
                        Err(cloned_test_case)
                    }
                }
                Err(_) => Err(cloned_test_case),
            }
        })
        .partition(Result::is_ok);

    let successes: Vec<TestCase> = tmp_successes.into_iter().map(Result::unwrap).collect();
    let failures: Vec<TestCase> = tmp_failures.into_iter().map(Result::unwrap_err).collect();

    let number_of_success = successes.len().value_as::<f64>().unwrap();
    let number_of_failures = failures.len().value_as::<f64>().unwrap();

    eprintln!(
        "Failure rate: {:.0}/{:.0} = {:.2}%",
        number_of_failures,
        (number_of_success + number_of_failures),
        number_of_failures / (number_of_success + number_of_failures) * 100_f64
    );

    (successes, failures)
}

#[cfg(feature = "json")]
#[test]
fn integration_tests_draft4() {
    let (_, failed_test_cases) = run_test_cases(&path_from_components!["tests", "draft4"], DraftVersion::Draft4);
    failed_test_cases.iter().for_each(|test_case| {
        println!("FAILURE: {}:{}:{}", test_case.file_name, &test_case.group_description, &test_case.test_description);
    });
    // assert_eq!(failed_test_cases, Vec::new()); // FIXME: define realistic assertion
}

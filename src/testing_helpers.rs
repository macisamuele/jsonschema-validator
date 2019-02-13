maybe_import_dependencies_for_parallel_run!();

use std::path::Path;
use std::path::PathBuf;
use url::Url;

#[macro_export]
macro_rules! expected_err {
    ($expression_to_match:expr, $expected_enum:path, $expected_check:expr) => {
        expected_err!($expression_to_match, $expected_enum, $expected_check,)
    };
    ($expression_to_match:expr, $expected_enum:path, $expected_check:expr,) => {
        let expression_result = $expression_to_match;
        if let Err($expected_enum(value)) = expression_result {
            $expected_check(value)
        } else {
            panic!("Expected {:?}, received {:?}", stringify![$expected_enum], expression_result);
        }
    };
}

#[macro_export]
macro_rules! expected_enum {
    ($enum_to_check:expr, $expected_enum:path) => {
        expected_enum!($enum_to_check, $expected_enum,)
    };
    ($enum_to_check:expr, $expected_enum:path,) => {
        if let $expected_enum = $enum_to_check {
        } else {
            panic!("Expected {:?}, received {:?}", stringify![$expected_enum], $enum_to_check,);
        }
    };
}

#[macro_export]
macro_rules! should_panic {
    ($block_to_panic:block) => {{
        let result = ::std::panic::catch_unwind(|| $block_to_panic);
        assert_eq!(result.is_err(), true);
    }};
}

fn root_repository_path() -> PathBuf {
    Path::new(file!()).canonicalize().unwrap().parent().unwrap().parent().unwrap().to_path_buf()
}

pub fn test_data_file_path(path: &str) -> String {
    String::from(
        path.split('/')
            .collect::<Vec<_>>()
            .iter()
            .fold(root_repository_path().join("test-data"), |iter_path, &path_path| iter_path.join(path_path))
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    )
}

pub fn test_data_file_url(path: &str) -> String {
    Url::from_file_path(&test_data_file_path(path)).unwrap().to_string()
}

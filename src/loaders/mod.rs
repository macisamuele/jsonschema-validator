#[macro_use]
mod loader_macros;
#[cfg(feature = "json")]
mod json;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
#[cfg(feature = "yaml")]
mod yaml;
#[cfg(feature = "json")]
pub use self::json::JSONLoader;
#[cfg(feature = "yaml")]
pub use self::yaml::YAMLLoader;

use crate::cache::Cache;
use crate::cache::Cached;
use crate::types::PrimitiveType;
pub use crate::url_helpers::normalize_url_for_cache;
pub use crate::url_helpers::parse_and_normalize_url;
pub use crate::url_helpers::UrlError;
use reqwest;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[derive(Debug, Display)]
pub enum LoaderError<FE> {
    IOError(io::Error),
    InvalidURL(UrlError),
    FetchURLFailed(reqwest::Error),
    FormatError(FE),
    UnknownError,
}

impl<FE> From<io::Error> for LoaderError<FE> {
    fn from(error: io::Error) -> Self {
        LoaderError::IOError(error)
    }
}

impl<FE> From<url::ParseError> for LoaderError<FE> {
    fn from(error: url::ParseError) -> Self {
        LoaderError::InvalidURL(UrlError::ParseError(error))
    }
}

impl<FE> From<url::SyntaxViolation> for LoaderError<FE> {
    fn from(error: url::SyntaxViolation) -> Self {
        LoaderError::InvalidURL(UrlError::SyntaxViolation(error))
    }
}

impl<FE> From<UrlError> for LoaderError<FE> {
    fn from(error: UrlError) -> Self {
        LoaderError::InvalidURL(error)
    }
}

impl<FE> From<reqwest::Error> for LoaderError<FE> {
    fn from(error: reqwest::Error) -> Self {
        LoaderError::FetchURLFailed(error)
    }
}

impl<FE> Default for LoaderError<FE> {
    #[inline]
    fn default() -> Self {
        LoaderError::UnknownError
    }
}

pub trait Loader<T>: Clone + Default + Debug + Sync + Send
where
    T: PrimitiveType<T>,
    LoaderError<Self::FormatError>: From<Self::FormatError>,
{
    type FormatError: Debug + Sync + Send;

    fn cache(&self) -> &Cache<Url, T>;

    fn load_from_string(content: String) -> Result<T, LoaderError<Self::FormatError>>;

    fn load<R: AsRef<str>>(&self, url: R) -> Result<T, LoaderError<Self::FormatError>> {
        self.load_with_timeout(url, Duration::from_millis(30_000))
    }

    fn load_with_timeout<R: AsRef<str>>(&self, url: R, timeout: Duration) -> Result<T, LoaderError<Self::FormatError>> {
        let url = parse_and_normalize_url(url)?;

        let normalized_url = normalize_url_for_cache(&url);

        let cached_value = {
            let thing: Result<Arc<T>, LoaderError<Self::FormatError>> = self.cache().get_or_fetch_with_result(&normalized_url, |url_to_fetch| {
                // Value was not available on cache
                Ok(Self::load_from_string(if url_to_fetch.scheme() == "file" {
                    fs::read_to_string(url_to_fetch.to_file_path().unwrap())?
                } else {
                    let client_builder = reqwest::Client::builder();
                    let client = client_builder.gzip(true).timeout(timeout).build()?;
                    client.get(url_to_fetch.as_ref()).send()?.error_for_status()?.text()?
                })?)
            });
            thing?
        };
        Ok(cached_value
            .fragment(url.fragment().unwrap())
            .and_then(|item| Some(item.clone()))
            .unwrap_or_else(T::default))
    }
}

#[cfg(any(test, feature = "testing"))]
#[macro_export]
macro_rules! mock_loader_request {
    ($loader:ident, $status_code:expr, $content_type:expr, $file_path:expr,) => {{
        let abs_file_path = test_data_file_path($file_path);
        let url_path = String::from(url::Url::parse(&test_data_file_url($file_path)).unwrap().path());
        let mocked_request = mock("GET", url_path.as_str())
            .with_status($status_code)
            .with_header("content-type", $content_type)
            .with_body_from_file(&abs_file_path)
            .create();
        let url = url::Url::parse(&server_url()).unwrap().join(url_path.as_str()).unwrap();

        let value = $loader.load(url);
        mocked_request.expect(1).assert();

        value
    }};
    ($loader:ident, $status_code:expr, $content_type:expr, $file_path:expr) => {{
        mock_loader_request!($loader, $status_code, $content_type, $file_path,)
    }};
    ($loader:ident, $status_code:expr, $file_path:expr,) => {{
        mock_loader_request!($loader, $status_code, "application/octet-stream", $file_path,)
    }};
    ($loader:ident, $status_code:expr, $file_path:expr) => {{
        mock_loader_request!($loader, $status_code, $file_path,)
    }};
    ($loader:ident, $file_path:expr,) => {{
        mock_loader_request!($loader, 200, $file_path,)
    }};
    ($loader:ident, $file_path:expr) => {{
        mock_loader_request!($loader, $file_path,)
    }};
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;
    use std::io;
    use test_case_derive::test_case;
    use url::Url;

    #[derive(Clone, Debug, Default)]
    struct TestLoader(Cache<Url, TestingType>);

    impl Loader<TestingType> for TestLoader {
        type FormatError = ();

        fn cache(&self) -> &Cache<Url, TestingType> {
            &self.0
        }

        fn load_from_string(content: String) -> Result<TestingType, LoaderError<()>> {
            let content = content.trim_end();
            if "ERR" == content {
                Err(())?
            } else {
                Ok(if content.is_empty() {
                    TestingType::Null
                } else if let Ok(value) = content.parse::<i32>() {
                    TestingType::from(value)
                } else if let Ok(value) = content.parse::<bool>() {
                    TestingType::from(value)
                } else {
                    TestingType::from(content)
                })
            }
        }
    }

    #[test]
    fn test_default_loader_error() {
        expected_enum!(LoaderError::<()>::default(), LoaderError::UnknownError);
    }

    #[test]
    fn test_load_wrong_url_parse_error() {
        expected_err!(TestLoader::default().load("this-is-a-wrong-url"), LoaderError::InvalidURL, |value| {
            expected_err!(
                {
                    let thing: Result<(), UrlError> = Err(value);
                    thing
                },
                UrlError::ParseError,
                |value| {
                    assert_eq!(value, url::ParseError::RelativeUrlWithoutBase);
                }
            );
        });
    }

    #[test]
    fn test_load_wrong_url_syntax_error() {
        expected_err!(TestLoader::default().load("http:/this-is-syntactically-invalid-url"), LoaderError::InvalidURL, |value| {
            expected_err!(
                {
                    let thing: Result<(), UrlError> = Err(value);
                    thing
                },
                UrlError::SyntaxViolation,
                |value| {
                    assert_eq!(value, url::SyntaxViolation::ExpectedDoubleSlash);
                }
            );
        });
    }

    #[test]
    fn test_load_from_not_existing_file() {
        let loader = TestLoader::default();
        let non_exiting_file_url = {
            let mut path = test_data_file_url("loader_tests/Null.txt");
            path.push_str("_Not_existing_anymore");
            path
        };
        expected_err!(loader.load(&non_exiting_file_url), LoaderError::IOError, |value: io::Error| {
            assert_eq!(value.kind(), io::ErrorKind::NotFound);
        });
    }

    #[test_case("loader_tests/Boolean.txt", TestingType::from(false))]
    #[test_case("loader_tests/Integer.txt", TestingType::from(1))]
    #[test_case("loader_tests/Null.txt", TestingType::from(()))]
    #[test_case("loader_tests/String.txt", TestingType::from("Some Text"))]
    fn test_load_from_file_valid_content(file_path: &str, expected_loaded_object: TestingType) {
        let mut loader = TestLoader::default();
        assert_eq!(TestLoader::default().load(&test_data_file_url(file_path)).ok().unwrap(), expected_loaded_object,);
    }

    #[test]
    fn test_load_from_file_invalid_content() {
        expected_err!(
            TestLoader::default().load(&test_data_file_url("loader_tests/Invalid.txt")),
            LoaderError::FormatError,
            |value| {
                assert_eq!(value, ());
            },
        );
    }

    #[test_case("loader_tests/Boolean.txt", TestingType::from(false))]
    #[test_case("loader_tests/Integer.txt", TestingType::from(1))]
    #[test_case("loader_tests/Null.txt", TestingType::from(()))]
    #[test_case("loader_tests/String.txt", TestingType::from("Some Text"))]
    fn test_load_from_url_valid_content(file_path: &str, expected_loaded_object: TestingType) {
        let mut loader = TestLoader::default();
        assert_eq!(mock_loader_request!(loader, file_path).unwrap(), expected_loaded_object);
    }

    #[test]
    fn test_load_from_url_invalid_content() {
        let loader = TestLoader::default();
        expected_err!(mock_loader_request!(loader, "loader_tests/Invalid.txt"), LoaderError::FormatError, |value| {
            assert_eq!(value, ());
        });
    }

    #[test]
    fn test_load_from_url_http_error() {
        let loader = TestLoader::default();

        expected_err!(
            mock_loader_request!(loader, 404, "loader_tests/Null.txt"),
            LoaderError::FetchURLFailed,
            |value: reqwest::Error| assert_eq!(value.status().and_then(|value| Some(value.as_u16())), Some(404)),
        );
    }
}

use crate::types::{keyword_type::KeywordType, validation_error::ValidationError};
use failure::Fail;
use loader_rs::LoaderError;
use url::{ParseError, Url};

#[derive(Debug, Fail)]
pub(in crate) enum SchemaError {
    #[fail(display = "Unknown error")]
    Unknown,
    #[fail(display = "Malformed Schema: path={}, detail={}", path, detail)]
    Malformed { path: Url, keyword: KeywordType, detail: String },
    #[fail(display = "Url Parsing error: {}", 0)]
    UrlParse(ParseError),
    #[fail(display = "Validation error: {}", 0)]
    Validation(ValidationError),
    #[fail(display = "Loader Error: {}", 0)]
    LoaderError(LoaderError),
}

impl Default for SchemaError {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<ParseError> for SchemaError {
    fn from(value: ParseError) -> Self {
        Self::UrlParse(value)
    }
}

impl From<ValidationError> for SchemaError {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

impl From<LoaderError> for SchemaError {
    fn from(value: LoaderError) -> Self {
        Self::LoaderError(value)
    }
}

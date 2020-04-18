use crate::types::keyword_type::KeywordType;
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub(in crate) struct ValidationError {
    // TODO: enhance content
    message: String,
    keyword: KeywordType,
    path: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

fn normalise_path(path: &str) -> &str {
    if path.eq("#/") {
        "#"
    } else if path.ends_with('/') {
        path.strip_suffix('/').unwrap()
    } else {
        path
    }
}

impl ValidationError {
    pub(in crate) fn new<IS: Into<String>>(path: &str, keyword: KeywordType, message: IS) -> Self {
        Self {
            path: normalise_path(path).to_string(),
            message: message.into(),
            keyword,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalise_path;
    use test_case::test_case;

    #[test_case("#"        => "#"       ; "Document root")]
    #[test_case("#/"       => "#"       ; "Document root, with additional /")]
    #[test_case("#/path1"  => "#/path1" ; "One level into document root")]
    #[test_case("#/path1/" => "#/path1" ; "One level into document root, with additional /")]
    fn path_normalisation(path_to_normalise: &str) -> &str {
        normalise_path(path_to_normalise)
    }
}

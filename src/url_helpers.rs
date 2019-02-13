#[cfg(feature = "regular_expression")]
use regex::Regex;
use std::cell::RefCell;
use url::ParseError;
use url::SyntaxViolation;
use url::Url;

#[derive(Clone, Debug, Display, PartialEq)]
pub enum UrlError {
    ParseError(ParseError),
    SyntaxViolation(SyntaxViolation),
    JsonFragmentError(String),
}

impl From<ParseError> for UrlError {
    fn from(error: ParseError) -> Self {
        UrlError::ParseError(error)
    }
}

impl From<SyntaxViolation> for UrlError {
    fn from(error: SyntaxViolation) -> Self {
        UrlError::SyntaxViolation(error)
    }
}

#[cfg(feature = "regular_expression")]
fn get_invalid_fragment_part_according_to_json_pointer_rules(url: &Url) -> Option<String> {
    // Checks https://tools.ietf.org/html/rfc6901 rules
    let regex = Regex::new("#.*(~([^01]|$))").unwrap();
    regex.captures(url.as_str()).and_then(|captures| Some(String::from(&captures[1])))
}

#[cfg(not(feature = "regular_expression"))]
fn get_invalid_fragment_part_according_to_json_pointer_rules(url: &Url) -> Option<String> {
    // Checks https://tools.ietf.org/html/rfc6901 rules
    // This check could have been done with a Regex but this requires regex dependency which adds ~20KB in the
    // final built library, which looks a bit too much as we are not using regex for anything else.
    //
    // As we're not sure about the real impact of having regex as dependency of this library
    // we're implementing a non-regex based solution, the code is a bit more verbose and so it's possible that it will
    // be selected a single solution.
    let fragment = url.fragment().unwrap_or("/");
    let mut next_character_error: Option<String> = None;

    let _ = fragment.chars().collect::<Vec<char>>().windows(2).any(|window| {
        let current = window[0];
        let next = window[1];

        if current == '~' && next != '0' && next != '1' {
            next_character_error = Some(format!("{}{}", current, next));
            return true;
        }
        false
    });

    if next_character_error.is_some() {
        next_character_error
    } else if fragment.ends_with('~') {
        Some("~".to_string())
    } else {
        None
    }
}

pub fn parse_and_normalize_url<R: AsRef<str>>(url: R) -> Result<Url, UrlError> {
    let syntax_violations = RefCell::new(Vec::<SyntaxViolation>::new());
    let mut url = Url::options()
        .syntax_violation_callback(Some(&|v| syntax_violations.borrow_mut().push(v)))
        .parse(url.as_ref())?;
    if let Some(violation) = syntax_violations.borrow().first() {
        Err(*violation)?
    }

    let cloned_url = url.clone();
    let fragments = cloned_url.fragment().unwrap_or("").trim_end_matches('/');

    let owned_fragment = if fragments.is_empty() {
        "/".to_string()
    } else if fragments.starts_with('/') {
        fragments.to_string()
    } else {
        format!("/{}", fragments)
    };

    url.set_fragment(Some(owned_fragment.as_str()));

    if let Some(invalid_fragment) = get_invalid_fragment_part_according_to_json_pointer_rules(&url) {
        return Err(UrlError::JsonFragmentError(invalid_fragment));
    }

    Ok(url)
}

pub fn normalize_url_for_cache(url: &Url) -> Url {
    let mut clone_url = url.clone();
    clone_url.set_fragment(Some("/"));
    clone_url
}

pub fn extract_fragment_components_from_fragment_string<R: AsRef<str>>(fragment_string: R) -> Vec<String> {
    let fragments: Vec<String> = fragment_string
        .as_ref()
        .split('/')
        .skip(1)
        .map(|fragment_part| fragment_part.replace("~1", "/").replace("~0", "~"))
        .collect();

    if fragments.len() == 1 && fragments[0] == "" {
        vec![]
    } else {
        fragments
    }
}

#[inline]
pub fn extract_fragment_components(url: &Url) -> Vec<String> {
    extract_fragment_components_from_fragment_string(url.fragment().unwrap_or("/"))
}

pub fn append_fragment_components<T: AsRef<str>, I: IntoIterator<Item = T>>(url: &Url, fragment_components: I) -> Url {
    let mut components = vec![String::from("")];
    components.extend(extract_fragment_components(url));
    components.extend(fragment_components.into_iter().map(|item| item.as_ref().to_string()));

    let mut cloned_url = url.clone();
    cloned_url.set_fragment(Some(
        components
            .iter()
            .map(|fragment_part| fragment_part.replace("~", "~0").replace("/", "~1"))
            .collect::<Vec<_>>()
            .join("/")
            .as_str(),
    ));
    cloned_url
}

#[cfg(test)]
mod tests {
    use super::append_fragment_components;
    use super::extract_fragment_components;
    use super::extract_fragment_components_from_fragment_string;
    use super::parse_and_normalize_url;
    use super::ParseError;
    use super::SyntaxViolation;
    use super::UrlError;
    use test_case_derive::test_case;

    #[test_case("memory://", "memory:///#/" :: "url_with_no_path_no_fragment")]
    #[test_case("memory://#", "memory:///#/" :: "url_with_no_path")]
    #[test_case("memory:///", "memory:///#/" :: "url_with_no_fragment")]
    #[test_case("memory:///#", "memory:///#/" :: "url_with_path_and_fragment")]
    #[test_case("memory:///#/", "memory:///#/" :: "url_with_path_and_fragment_normalized")]
    #[test_case("memory:///#fragment", "memory:///#/fragment" :: "url_with_path_and_not_empty_fragment_1")]
    #[test_case("memory:///#/fragment", "memory:///#/fragment" :: "url_with_path_and_not_empty_fragment_2")]
    #[test_case("memory:///#/fragment/", "memory:///#/fragment" :: "url_with_path_and_not_empty_fragment_3")]
    fn test_parse_and_normalize_url_valid_case(url_str: &str, expected_result_str: &str) {
        assert_eq!(parse_and_normalize_url(url_str).unwrap().as_str(), expected_result_str,);
    }

    #[test_case("http:///", UrlError::ParseError(ParseError::EmptyHost))]
    #[test_case("http://300.0.0.0/", UrlError::ParseError(ParseError::InvalidIpv4Address))]
    #[test_case("memory://#/\0a", UrlError::SyntaxViolation(SyntaxViolation::NullInFragment))]
    #[test_case("http:/example", UrlError::SyntaxViolation(SyntaxViolation::ExpectedDoubleSlash))]
    #[test_case("memory://#/~", UrlError::JsonFragmentError(String::from("~")))]
    #[test_case("memory://#/~a", UrlError::JsonFragmentError(String::from("~a")))]
    #[test_case("memory://#/~0/~1/~c", UrlError::JsonFragmentError(String::from("~c")))]
    fn test_parse_and_normalize_url_invalid_case(url_str: &str, expected_err: UrlError) {
        assert_eq!(parse_and_normalize_url(url_str).unwrap_err(), expected_err);
    }

    #[allow(clippy::redundant_closure)]
    #[test_case("", vec![] :: "empty fragment")]
    #[test_case("#", vec![] :: "Only pound in fragment")]
    #[test_case("/", vec![] :: "Only slash in fragment")]
    #[test_case("#/", vec![] :: "Pound and slash in fragment")]
    #[test_case("/a/~0/b/~1/c", vec!["a", "~", "b", "/", "c"] :: "Complex fragment not starting with pound")]
    #[test_case("#/a/~0/b/~1/c", vec!["a", "~", "b", "/", "c"] :: "Complex fragment starting with pound")]
    fn test_extract_fragment_components_from_fragment_string(url_str: &str, expected_result: Vec<&str>) {
        assert_eq!(
            extract_fragment_components_from_fragment_string(url_str),
            expected_result.iter().map(|part: &&str| part.to_string()).collect::<Vec<String>>()
        );
    }

    #[allow(clippy::redundant_closure)]
    #[test_case("memory://", vec![] :: "Bare minimal url")]
    #[test_case("memory://#/", vec![] :: "Minimal url with fragment")]
    #[test_case("memory://#/a/~0/b/~1/c", vec!["a", "~", "b", "/", "c"] :: "Complex fragment")]
    fn test_extract_fragment_components(url_str: &str, expected_result: Vec<&str>) {
        assert_eq!(
            extract_fragment_components(&parse_and_normalize_url(url_str).ok().unwrap()),
            expected_result.iter().map(|part: &&str| part.to_string()).collect::<Vec<String>>()
        );
    }

    #[test_case("memory://", vec![], "memory:///#")]
    #[test_case("memory://", vec!["a"], "memory:///#/a")]
    #[test_case("memory://#/a/b", vec!["c"], "memory:///#/a/b/c")]
    fn test_append_fragment_components(url_str: &str, components_to_add: Vec<&str>, expected_result: &str) {
        assert_eq!(
            append_fragment_components(&parse_and_normalize_url(url_str).ok().unwrap(), components_to_add).as_str(),
            expected_result,
        );
    }
}

maybe_import_dependencies_for_parallel_run!();

use super::EnumPrimitiveType;
use super::Index;
use super::JsonMap;
use super::JsonMapTrait;
use crate::url_helpers::extract_fragment_components_from_fragment_string;
use std::fmt::Debug;

pub trait PrimitiveType<T>
where
    Self: JsonMapTrait<T> + Clone + Debug + Default + PartialEq + Sync + Send,
    T: PrimitiveType<T>,
{
    fn has_attribute(&self, attribute_name: &str) -> bool {
        self.get_attribute(attribute_name).is_some()
    }

    #[inline]
    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    #[inline]
    fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    #[inline]
    fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    #[inline]
    fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    #[inline]
    fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    #[inline]
    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    #[inline]
    fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    // This trait allows us to have a 1:1 mapping with serde_json, generally used by rust libraries
    // but gives us the power to use different objects from serde_json. This gives us the ability
    // to support usage of different data-types like PyObject from pyo3 in case of python bindings
    fn primitive_type(&self) -> Option<EnumPrimitiveType> {
        // This might not be efficient, but it could be comfortable to quickly extract the type especially while debugging
        if self.is_array() {
            Some(EnumPrimitiveType::Array)
        } else if self.is_boolean() {
            Some(EnumPrimitiveType::Boolean)
        } else if self.is_integer() {
            Some(EnumPrimitiveType::Integer)
        } else if self.is_null() {
            Some(EnumPrimitiveType::Null)
        } else if self.is_number() {
            Some(EnumPrimitiveType::Number)
        } else if self.is_object() {
            Some(EnumPrimitiveType::Object)
        } else if self.is_string() {
            Some(EnumPrimitiveType::String)
        } else {
            None
        }
    }

    fn get_attribute<R: AsRef<str>>(&self, attribute_name: R) -> Option<&Self>;
    fn get_index(&self, index: usize) -> Option<&Self>;

    fn get<I: Index<Self>>(&self, index: I) -> Option<&Self>
    where
        Self: PrimitiveType<Self>,
    {
        index.index_into(self)
    }

    fn as_array(&self) -> Option<Vec<&T>>;
    fn as_boolean(&self) -> Option<bool>;
    fn as_integer(&self) -> Option<i128>;
    fn as_null(&self) -> Option<()>;
    fn as_number(&self) -> Option<f64>; // TODO: deal with f128
    fn as_object(&self) -> Option<JsonMap<T>>;
    fn as_string(&self) -> Option<&str>;
    fn fragment<R: AsRef<str>>(&self, fragment: R) -> Option<&Self> {
        // NOTE: Iteration order matters, so iterate![] should not be used here
        extract_fragment_components_from_fragment_string(fragment)
            .iter()
            // Using fold as for now tail recursion is not supported in rust, but if it will ever happen then `fold` will most probably be the first candidate
            .fold(Some(self), |result, fragment_part| {
                result.and_then(
                    |value| {
                        match value.primitive_type() {
                            Some(EnumPrimitiveType::Object) => value.get_attribute(fragment_part),
                            Some(EnumPrimitiveType::Array) => fragment_part.parse::<usize>().and_then(|index| {Ok(value.get_index(index))}).ok().unwrap_or(None),
                            _ => None,
                        }
                    }
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::testing_prelude::*;
    use crate::url_helpers::parse_and_normalize_url;
    use test_case_derive::test_case;

    #[test_case("memory:///#", Some(&testing_map![
        "key" => testing_map![
            "inner_key" => testing_vec![
                1,
                "2"
            ],
        ],
    ]))]
    #[test_case("memory:///#/key", Some(&testing_map![
        "inner_key" => testing_vec![
            1,
            "2"
        ],
    ]))]
    #[test_case("memory:///#/key/inner_key", Some(&testing_vec![
        1,
        "2"
    ]))]
    #[test_case("memory:///#/key/inner_key/0", Some(&TestingType::from(1)))]
    #[test_case("memory:///#/key/inner_key/1", Some(&TestingType::from("2")))]
    #[test_case("memory:///#/not_present", None)]
    #[test_case("memory:///#/key/inner_key/a", None)]
    #[test_case("memory:///#/key/inner_key/2", None)]
    fn test_get_fragment(url_str: &str, expected_value: Option<&TestingType>) {
        let external_map = testing_map![
            "key" => testing_map![
                "inner_key" => testing_vec![
                    1,
                    "2"
                ],
            ],
        ];
        assert_eq!(external_map.fragment(parse_and_normalize_url(url_str).unwrap().fragment().unwrap()), expected_value);
    }
}

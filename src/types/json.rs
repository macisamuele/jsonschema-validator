maybe_import_dependencies_for_parallel_run!();

use crate::types::JsonMap;
use crate::types::JsonMapTrait;
use crate::types::PrimitiveType;
use serde_json;

impl JsonMapTrait<serde_json::Value> for serde_json::Value {
    #[inline]
    fn keys(&self) -> Option<Vec<&str>>
    where
        serde_json::Value: 'static,
    {
        match self.as_object() {
            None => None,
            Some(obj) => Some(obj.keys().map(String::as_str).collect()),
        }
    }

    #[inline]
    fn values(&self) -> Option<Vec<&Self>> {
        match self.as_object() {
            None => None,
            Some(obj) => Some(obj.values().collect()),
        }
    }

    #[inline]
    fn items(&self) -> Option<Vec<(&str, &Self)>> {
        match self.as_object() {
            None => None,
            Some(obj) => Some(obj.iter() // TODO: check if possible to use parallel iterator (maybe a small PR to serde-json is needed)
                .map(|(key, value)| (key.as_str(), value))
                .collect()),
        }
    }
}

impl PrimitiveType<serde_json::Value> for serde_json::Value {
    fn has_attribute(&self, attribute_name: &str) -> bool {
        self.get(attribute_name).is_some()
    }

    fn get_attribute<R: AsRef<str>>(&self, attribute_name: R) -> Option<&Self> {
        self.get(attribute_name.as_ref())
    }

    fn get_index(&self, index: usize) -> Option<&Self> {
        self.get(index)
    }

    fn as_array(&self) -> Option<Vec<&Self>> {
        match self.as_array() {
            Some(array) => Some(iterate![array].map(|value: &Self| value).collect()),
            None => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        self.as_bool()
    }

    fn as_integer(&self) -> Option<i128> {
        match self.as_i64() {
            Some(t) => Some(i128::from(t)),
            None => None,
        }
    }

    fn as_null(&self) -> Option<()> {
        self.as_null()
    }

    fn as_number(&self) -> Option<f64> {
        self.as_f64() // TODO: deal with f128
    }

    fn as_object(&self) -> Option<JsonMap<Self>> {
        // FIXME: check that this writing is correct, if it is update the others None => None otherwise fix this
        let _ = self.as_object()?;
        Some(JsonMap::new(self))
    }

    fn as_string(&self) -> Option<&str> {
        self.as_str()
    }
}

#[cfg(test)]
mod tests_json_map_trait {
    use crate::testing_prelude::*;

    lazy_static! {
        static ref TESTING_MAP: serde_json::Value = json![{"k1": "v1", "k2": "v2"}];
        static ref TESTING_LIST: serde_json::Value = json![["k1", "k2"]];
    }

    #[test]
    fn keys_for_map() {
        let testing_map: &serde_json::Value = &TESTING_MAP;
        assert_eq!(JsonMap::new(testing_map).keys(), Some(vec!["k1", "k2"]));
    }

    #[test]
    fn keys_for_non_map() {
        let testing_list: &serde_json::Value = &TESTING_LIST;
        assert_eq!(JsonMap::new(testing_list).keys(), None);
    }

    #[test]
    fn values_for_map() {
        let testing_map: &serde_json::Value = &TESTING_MAP;
        assert_eq!(JsonMap::new(testing_map).values(), Some(vec![&json!["v1"], &json!["v2"]]));
    }

    #[test]
    fn values_for_non_map() {
        let testing_list: &serde_json::Value = &TESTING_LIST;
        assert_eq!(JsonMap::new(testing_list).values(), None);
    }

    #[test]
    fn items_for_map() {
        let testing_map: &serde_json::Value = &TESTING_MAP;
        assert_eq!(JsonMap::new(testing_map).items(), Some(vec![("k1", &json!["v1"]), ("k2", &json!["v2"])]));
    }

    #[test]
    fn items_for_non_map() {
        let testing_list: &serde_json::Value = &TESTING_LIST;
        assert_eq!(JsonMap::new(testing_list).items(), None);
    }
}

#[cfg(test)]
mod tests_primitive_type_trait {
    use crate::testing_prelude::*;
    use test_case_derive::test_case;

    #[test_case(json![[]], EnumPrimitiveType::Array)]
    #[test_case(json![true], EnumPrimitiveType::Boolean)]
    #[test_case(json![1], EnumPrimitiveType::Integer)]
    #[test_case(json![null], EnumPrimitiveType::Null)]
    #[test_case(json![1.2], EnumPrimitiveType::Number)]
    #[test_case(json![{"prop": "value"}], EnumPrimitiveType::Object)]
    #[test_case(json!["string"], EnumPrimitiveType::String)]
    fn test_primitive_type(value: serde_json::Value, expected_value: EnumPrimitiveType) {
        assert_eq!(PrimitiveType::primitive_type(&value), Some(expected_value))
    }

    #[test_case(json![{"present": 1}], "present", Some(&json![1]))]
    #[test_case(json![{"present": 1}], "not-present", None)]
    fn test_get_attribute(value: serde_json::Value, attribute_name: &str, expected_value: Option<&serde_json::Value>) {
        assert_eq!(PrimitiveType::get_attribute(&value, attribute_name), expected_value);
    }

    #[test_case(json![[0, 1, 2]], 1, Some(&json![1]))]
    #[test_case(json![[0, 1, 2]], 4, None)]
    fn test_get_index(value: serde_json::Value, index: usize, expected_value: Option<&serde_json::Value>) {
        assert_eq!(PrimitiveType::get_index(&value, index), expected_value);
    }

    #[test_case(json![{"present": 1}], "present", Some(&json![1]))]
    #[test_case(json![{"present": 1}], "not-present", None)]
    #[test_case(json![[0, 1, 2]], 1, Some(&json![1]))]
    #[test_case(json![[0, 1, 2]], 4, None)]
    fn test_get<I: Index<serde_json::Value>>(value: serde_json::Value, index_value: I, expected_value: Option<&serde_json::Value>) {
        assert_eq!(PrimitiveType::get(&value, index_value), expected_value);
    }

    #[test_case(json![{"present": 1}], "present", true)]
    #[test_case(json![{"present": 1}], "not-present", false)]
    #[test_case(json![[1, 2, 3]], "not-present", false)]
    fn test_has_attribute(value: serde_json::Value, attr_name: &str, expected_value: bool) {
        assert_eq!(PrimitiveType::has_attribute(&value, attr_name), expected_value);
    }

    #[test_case(json![[0, 1, 2]], true)]
    #[test_case(json![true], false)]
    #[test_case(json![1_u32], false)]
    #[test_case(json![null], false)]
    #[test_case(json![1.2_f32], false)]
    #[test_case(json![{"key": "value"}], false)]
    #[test_case(json!["string"], false)]
    fn test_is_array(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_array(&value), expected_value);
    }

    #[test_case(json![[0, 1, 2]], false)]
    #[test_case(json![true], true)]
    #[test_case(json![1_u32], false)]
    #[test_case(json![null], false)]
    #[test_case(json![1.2_f32], false)]
    #[test_case(json![{"key": "value"}], false)]
    #[test_case(json!["string"], false)]
    fn test_is_boolean(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_boolean(&value), expected_value);
    }

    #[test_case(json![[0, 1, 2]], false)]
    #[test_case(json![true], false)]
    #[test_case(json![1_u32], true)]
    #[test_case(json![null], false)]
    #[test_case(json![1.2_f32], false)]
    #[test_case(json![{"key": "value"}], false)]
    #[test_case(json!["string"], false)]
    fn test_is_integer(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_integer(&value), expected_value);
    }

    #[test_case(json![[0, 1, 2]], false)]
    #[test_case(json![true], false)]
    #[test_case(json![1_u32], false)]
    #[test_case(json![null], true)]
    #[test_case(json![1.2_f32], false)]
    #[test_case(json![{"key": "value"}], false)]
    #[test_case(json!["string"], false)]
    fn test_is_null(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_null(&value), expected_value);
    }

    #[test_case(json![[0, 1, 2]], false)]
    #[test_case(json![true], false)]
    #[test_case(json![1_u32], true)]
    #[test_case(json![null], false)]
    #[test_case(json![1.2_f32], true)]
    #[test_case(json![{"key": "value"}], false)]
    #[test_case(json!["string"], false)]
    fn test_is_number(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_number(&value), expected_value);
    }

    #[test_case(json![[0, 1, 2]], false)]
    #[test_case(json![true], false)]
    #[test_case(json![1_u32], false)]
    #[test_case(json![null], false)]
    #[test_case(json![1.2_f32], false)]
    #[test_case(json![{"key": "value"}], true)]
    #[test_case(json!["string"], false)]
    fn test_is_object(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_object(&value), expected_value);
    }

    #[test_case(json![[0, 1, 2]], false)]
    #[test_case(json![true], false)]
    #[test_case(json![1_u32], false)]
    #[test_case(json![null], false)]
    #[test_case(json![1.2_f32], false)]
    #[test_case(json![{"key": "value"}], false)]
    #[test_case(json!["string"], true)]
    fn test_is_string(value: serde_json::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_string(&value), expected_value);
    }

    #[test_case(json![[1]], Some(vec![&json![1]]))]
    #[test_case(json![[1, "a"]], Some(vec![&json![1], &json!["a"]]))]
    #[test_case(json![null], None)]
    fn test_as_array(value: serde_json::Value, expected_value: Option<Vec<&serde_json::Value>>) {
        assert_eq!(PrimitiveType::as_array(&value), expected_value);
    }

    #[test_case(json![true], Some(true))]
    #[test_case(json![false], Some(false))]
    #[test_case(json![1], None)]
    fn test_as_boolean(value: serde_json::Value, expected_value: Option<bool>) {
        assert_eq!(PrimitiveType::as_boolean(&value), expected_value);
    }

    #[test_case(json![1], Some(1))]
    #[test_case(json![1.2], None)]
    #[test_case(json!["1"], None)]
    fn test_as_integer(value: serde_json::Value, expected_value: Option<i128>) {
        assert_eq!(PrimitiveType::as_integer(&value), expected_value);
    }

    #[test_case(json![null], Some(()))]
    #[test_case(json!["1"], None)]
    fn test_as_null(value: serde_json::Value, expected_value: Option<()>) {
        assert_eq!(PrimitiveType::as_null(&value), expected_value);
    }

    #[test_case(json![1], Some(1_f64))]
    #[test_case(json![1.2], Some(1.2))]
    #[test_case(json!["1"], None)]
    fn test_as_number(value: serde_json::Value, expected_value: Option<f64>) {
        assert_eq!(PrimitiveType::as_number(&value), expected_value);
    }

    #[test_case(json![1], None)]
    #[test_case(json![1.2], None)]
    #[test_case(json![{"1": 1}], Some(&json![{"1": 1}]))]
    fn test_as_object(value: serde_json::Value, expected_value: Option<&serde_json::Value>) {
        use std::ops::Deref;

        let option_as_object = PrimitiveType::as_object(&value);

        assert_eq!(option_as_object.is_some(), expected_value.is_some());

        if let Some(as_object) = option_as_object {
            assert_eq!(as_object.deref(), expected_value.unwrap());
        }
    }

    #[test_case(json![1], None)]
    #[test_case(json![1.2], None)]
    #[test_case(json!["1"], Some("1"))]
    fn test_as_string(value: serde_json::Value, expected_value: Option<&str>) {
        assert_eq!(PrimitiveType::as_string(&value), expected_value);
    }
}

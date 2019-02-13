maybe_import_dependencies_for_parallel_run!();

use crate::types::JsonMap;
use crate::types::JsonMapTrait;
use crate::types::PrimitiveType;
use serde_yaml;

impl JsonMapTrait<serde_yaml::Value> for serde_yaml::Value {
    #[inline]
    fn keys(&self) -> Option<Vec<&str>>
    where
        serde_yaml::Value: 'static,
    {
        match self.as_mapping() {
            None => None,
            Some(obj) => Some(obj.iter().filter_map(|(key, _)| key.as_str()).collect()),
        }
    }

    #[inline]
    fn values(&self) -> Option<Vec<&Self>> {
        match self.as_mapping() {
            None => None,
            Some(obj) => Some(obj.iter().map(|(_, value)| value).collect()),
        }
    }

    #[inline]
    fn items(&self) -> Option<Vec<(&str, &Self)>> {
        match self.as_mapping() {
            None => None,
            Some(obj) => Some(
                obj.iter() // TODO: check if possible to use parallel iterator (maybe a small PR to serde-yaml is needed)
                    .filter_map(|(key, value)| match key.as_str() {
                        None => None,
                        Some(k) => Some((k, value)),
                    })
                    .collect(),
            ),
        }
    }
}

impl PrimitiveType<serde_yaml::Value> for serde_yaml::Value {
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
        match self.as_sequence() {
            Some(array) => Some(iterate![array].map(|value: &Self| value).collect()),
            None => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        self.as_bool()
    }

    fn as_integer(&self) -> Option<i128> {
        match self.as_i64() {
            Some(n) => Some(i128::from(n)),
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
        let _ = self.as_mapping()?;
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
        static ref TESTING_MAP: serde_yaml::Value = yaml![{"k1": "v1", "k2": "v2"}];
        static ref TESTING_LIST: serde_yaml::Value = yaml![["k1", "k2"]];
    }

    #[test]
    fn keys_for_map() {
        let testing_map: &serde_yaml::Value = &TESTING_MAP;
        assert_eq!(JsonMap::new(testing_map).keys(), Some(vec!["k1", "k2"]));
    }

    #[test]
    fn keys_for_non_map() {
        let testing_list: &serde_yaml::Value = &TESTING_LIST;
        assert_eq!(JsonMap::new(testing_list).keys(), None);
    }

    #[test]
    fn values_for_map() {
        let testing_map: &serde_yaml::Value = &TESTING_MAP;
        assert_eq!(JsonMap::new(testing_map).values(), Some(vec![&yaml!["v1"], &yaml!["v2"]]));
    }

    #[test]
    fn values_for_non_map() {
        let testing_list: &serde_yaml::Value = &TESTING_LIST;
        assert_eq!(JsonMap::new(testing_list).values(), None);
    }

    #[test]
    fn items_for_map() {
        let testing_map: &serde_yaml::Value = &TESTING_MAP;
        assert_eq!(JsonMap::new(testing_map).items(), Some(vec![("k1", &yaml!["v1"]), ("k2", &yaml!["v2"])]));
    }

    #[test]
    fn items_for_non_map() {
        let testing_list: &serde_yaml::Value = &TESTING_LIST;
        assert_eq!(JsonMap::new(testing_list).items(), None);
    }
}

#[cfg(test)]
mod tests_primitive_type_trait {
    use crate::testing_prelude::*;
    use test_case_derive::test_case;

    #[test_case(yaml![[]], EnumPrimitiveType::Array)]
    #[test_case(yaml![true], EnumPrimitiveType::Boolean)]
    #[test_case(yaml![1], EnumPrimitiveType::Integer)]
    #[test_case(yaml![null], EnumPrimitiveType::Null)]
    #[test_case(yaml![1.2], EnumPrimitiveType::Number)]
    #[test_case(yaml![{"prop": "value"}], EnumPrimitiveType::Object)]
    #[test_case(yaml!["string"], EnumPrimitiveType::String)]
    fn test_primitive_type(value: serde_yaml::Value, expected_value: EnumPrimitiveType) {
        assert_eq!(PrimitiveType::primitive_type(&value), Some(expected_value))
    }

    #[test_case(yaml![{"present": 1}], "present", Some(&yaml![1]))]
    #[test_case(yaml![{"present": 1}], "not-present", None)]
    fn test_get_attribute(value: serde_yaml::Value, attribute_name: &str, expected_value: Option<&serde_yaml::Value>) {
        assert_eq!(PrimitiveType::get_attribute(&value, attribute_name), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], 1, Some(&yaml![1]))]
    #[test_case(yaml![[0, 1, 2]], 4, None)]
    fn test_get_index(value: serde_yaml::Value, index: usize, expected_value: Option<&serde_yaml::Value>) {
        assert_eq!(PrimitiveType::get_index(&value, index), expected_value);
    }

    #[test_case(yaml![{"present": 1}], "present", Some(&yaml![1]))]
    #[test_case(yaml![{"present": 1}], "not-present", None)]
    #[test_case(yaml![[0, 1, 2]], 1, Some(&yaml![1]))]
    #[test_case(yaml![[0, 1, 2]], 4, None)]
    fn test_get<I: Index<serde_yaml::Value>>(value: serde_yaml::Value, index_value: I, expected_value: Option<&serde_yaml::Value>) {
        assert_eq!(PrimitiveType::get(&value, index_value), expected_value);
    }

    #[test_case(yaml![{"present": 1}], "present", true)]
    #[test_case(yaml![{"present": 1}], "not-present", false)]
    #[test_case(yaml![[1, 2, 3]], "not-present", false)]
    fn test_has_attribute(value: serde_yaml::Value, attr_name: &str, expected_value: bool) {
        assert_eq!(PrimitiveType::has_attribute(&value, attr_name), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], true)]
    #[test_case(yaml![true], false)]
    #[test_case(yaml![1_u32], false)]
    #[test_case(yaml![null], false)]
    #[test_case(yaml![1.2_f32], false)]
    #[test_case(yaml![{"key": "value"}], false)]
    #[test_case(yaml!["string"], false)]
    fn test_is_array(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_array(&value), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], false)]
    #[test_case(yaml![true], true)]
    #[test_case(yaml![1_u32], false)]
    #[test_case(yaml![null], false)]
    #[test_case(yaml![1.2_f32], false)]
    #[test_case(yaml![{"key": "value"}], false)]
    #[test_case(yaml!["string"], false)]
    fn test_is_boolean(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_boolean(&value), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], false)]
    #[test_case(yaml![true], false)]
    #[test_case(yaml![1_u32], true)]
    #[test_case(yaml![null], false)]
    #[test_case(yaml![1.2_f32], false)]
    #[test_case(yaml![{"key": "value"}], false)]
    #[test_case(yaml!["string"], false)]
    fn test_is_integer(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_integer(&value), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], false)]
    #[test_case(yaml![true], false)]
    #[test_case(yaml![1_u32], false)]
    #[test_case(yaml![null], true)]
    #[test_case(yaml![1.2_f32], false)]
    #[test_case(yaml![{"key": "value"}], false)]
    #[test_case(yaml!["string"], false)]
    fn test_is_null(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_null(&value), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], false)]
    #[test_case(yaml![true], false)]
    #[test_case(yaml![1_u32], true)]
    #[test_case(yaml![null], false)]
    #[test_case(yaml![1.2_f32], true)]
    #[test_case(yaml![{"key": "value"}], false)]
    #[test_case(yaml!["string"], false)]
    fn test_is_number(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_number(&value), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], false)]
    #[test_case(yaml![true], false)]
    #[test_case(yaml![1_u32], false)]
    #[test_case(yaml![null], false)]
    #[test_case(yaml![1.2_f32], false)]
    #[test_case(yaml![{"key": "value"}], true)]
    #[test_case(yaml!["string"], false)]
    fn test_is_object(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_object(&value), expected_value);
    }

    #[test_case(yaml![[0, 1, 2]], false)]
    #[test_case(yaml![true], false)]
    #[test_case(yaml![1_u32], false)]
    #[test_case(yaml![null], false)]
    #[test_case(yaml![1.2_f32], false)]
    #[test_case(yaml![{"key": "value"}], false)]
    #[test_case(yaml!["string"], true)]
    fn test_is_string(value: serde_yaml::Value, expected_value: bool) {
        assert_eq!(PrimitiveType::is_string(&value), expected_value);
    }

    #[test_case(yaml![[1]], Some(vec![&yaml![1]]))]
    #[test_case(yaml![[1, "a"]], Some(vec![&yaml![1], &yaml!["a"]]))]
    #[test_case(yaml![null], None)]
    fn test_as_array(value: serde_yaml::Value, expected_value: Option<Vec<&serde_yaml::Value>>) {
        assert_eq!(PrimitiveType::as_array(&value), expected_value);
    }

    #[test_case(yaml![true], Some(true))]
    #[test_case(yaml![false], Some(false))]
    #[test_case(yaml![1], None)]
    fn test_as_boolean(value: serde_yaml::Value, expected_value: Option<bool>) {
        assert_eq!(PrimitiveType::as_boolean(&value), expected_value);
    }

    #[test_case(yaml![1], Some(1))]
    #[test_case(yaml![1.2], None)]
    #[test_case(yaml!["1"], None)]
    fn test_as_integer(value: serde_yaml::Value, expected_value: Option<i128>) {
        assert_eq!(PrimitiveType::as_integer(&value), expected_value);
    }

    #[test_case(yaml![null], Some(()))]
    #[test_case(yaml!["1"], None)]
    fn test_as_null(value: serde_yaml::Value, expected_value: Option<()>) {
        assert_eq!(PrimitiveType::as_null(&value), expected_value);
    }

    #[test_case(yaml![1], Some(1_f64))]
    #[test_case(yaml![1.2], Some(1.2))]
    #[test_case(yaml!["1"], None)]
    fn test_as_number(value: serde_yaml::Value, expected_value: Option<f64>) {
        assert_eq!(PrimitiveType::as_number(&value), expected_value);
    }

    #[test_case(yaml![1], None)]
    #[test_case(yaml![1.2], None)]
    #[test_case(yaml![{"1": 1}], Some(&yaml![{"1": 1}]))]
    fn test_as_object(value: serde_yaml::Value, expected_value: Option<&serde_yaml::Value>) {
        use std::ops::Deref;

        let option_as_object = PrimitiveType::as_object(&value);

        assert_eq!(option_as_object.is_some(), expected_value.is_some());

        if let Some(as_object) = option_as_object {
            assert_eq!(as_object.deref(), expected_value.unwrap());
        }
    }

    #[test_case(yaml![1], None)]
    #[test_case(yaml![1.2], None)]
    #[test_case(yaml!["1"], Some("1"))]
    fn test_as_string(value: serde_yaml::Value, expected_value: Option<&str>) {
        assert_eq!(PrimitiveType::as_string(&value), expected_value);
    }
}

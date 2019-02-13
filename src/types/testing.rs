maybe_import_dependencies_for_parallel_run!();

use crate::types::JsonMap;
use crate::types::JsonMapTrait;
use crate::types::PrimitiveType;
use std::collections::HashMap;

#[macro_export]
macro_rules! testing_map {
    ($($k:expr => $v: expr),*,) => {{
        testing_map![$($k => $v),*]
    }};
    ($($k: expr => $v: expr),*) => {{
        use crate::testing_prelude::*;

        // Variable definition is needed to ensure that the resulting type is known in the context
        #[allow(unused_mut)]
        let mut thing: HashMap<String, TestingType> = HashMap::default();
        $( let _ = thing.insert($k.to_string(), TestingType::from($v)); )*
        TestingType::from(thing)
    }};
}

#[macro_export]
macro_rules! testing_vec {
    ($($item: expr),*,) => {{
        testing_vec![$($item),*]
    }};
    ($($item: expr),*) => {{
        use crate::testing_prelude::*;

        // Variable definition is needed to ensure that the resulting type is known in the context
        let thing: Vec<TestingType> = vec![
            $( TestingType::from($item), )*
        ];
        TestingType::from(thing)
    }};
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestingType {
    Null,
    Boolean(bool),
    String(String),
    Integer(i32),
    List(Vec<TestingType>),
    Object(HashMap<String, TestingType>),
}

impl Default for TestingType {
    fn default() -> Self {
        TestingType::Null
    }
}

impl From<()> for TestingType {
    fn from(_: ()) -> Self {
        TestingType::Null
    }
}

impl From<bool> for TestingType {
    fn from(value: bool) -> Self {
        TestingType::Boolean(value)
    }
}

impl From<&str> for TestingType {
    fn from(value: &str) -> Self {
        TestingType::String(String::from(value))
    }
}

impl From<String> for TestingType {
    fn from(value: String) -> Self {
        TestingType::String(value)
    }
}

impl From<i32> for TestingType {
    fn from(value: i32) -> Self {
        TestingType::Integer(value)
    }
}

impl From<HashMap<String, TestingType>> for TestingType {
    fn from(value: HashMap<String, Self>) -> Self {
        TestingType::Object(value)
    }
}

impl From<Vec<TestingType>> for TestingType {
    fn from(value: Vec<Self>) -> Self {
        TestingType::List(value)
    }
}

impl JsonMapTrait<TestingType> for TestingType {
    fn items(&self) -> Option<Vec<(&str, &Self)>> {
        match self {
            TestingType::Object(hash_map) => Some(iterate![hash_map].map(|(k, v)| (k.as_str(), v)).collect()),
            _ => None,
        }
    }
}

impl PrimitiveType<TestingType> for TestingType {
    fn get_attribute<R: AsRef<str>>(&self, attribute_name: R) -> Option<&Self> {
        if let TestingType::Object(object) = self {
            object.get(attribute_name.as_ref())
        } else {
            None
        }
    }

    fn get_index(&self, index: usize) -> Option<&Self> {
        if let TestingType::List(array) = self {
            array.get(index)
        } else {
            None
        }
    }

    fn as_array(&self) -> Option<Vec<&Self>> {
        match self {
            TestingType::List(v) => Some(iterate![v].collect()),
            _ => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match self {
            TestingType::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<i128> {
        match self {
            TestingType::Integer(v) => Some(i128::from(*v)),
            _ => None,
        }
    }

    fn as_null(&self) -> Option<()> {
        match self {
            TestingType::Null => Some(()),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            TestingType::Integer(v) => Some(f64::from(*v)),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<JsonMap<Self>> {
        match self {
            TestingType::Object(v) => Some(JsonMap::new(&Self::from(v.clone()))),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            TestingType::String(s) => Some(s),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "yaml"))]
#[macro_export]
macro_rules! yaml {
    ($($json:tt)+) => {{
        use serde_json;
        use serde_yaml;
        let thing: serde_yaml::Value = serde_yaml::from_str(
            serde_json::to_string(&json![$($json)+]).unwrap().as_str(),
        ).unwrap();
        thing
    }};
}

#[cfg(test)]
mod smoke_test {
    use crate::testing_prelude::*;

    #[test]
    fn test_testing_type_instance_string() {
        let string = "string";
        let testing_type_instance = TestingType::from(string);
        assert_eq!(testing_type_instance.as_string(), Some(string));
        assert_eq!(testing_type_instance.has_attribute("attribute"), false);
        assert_eq!(testing_type_instance.is_array(), false);
        assert_eq!(testing_type_instance.is_boolean(), false);
        assert_eq!(testing_type_instance.is_integer(), false);
        assert_eq!(testing_type_instance.is_null(), false);
        assert_eq!(testing_type_instance.is_number(), false);
        assert_eq!(testing_type_instance.is_object(), false);
        assert_eq!(testing_type_instance.is_string(), true);
    }

    #[test]
    fn test_testing_type_instance_integer() {
        let integer = 1;
        let testing_type_instance = TestingType::from(integer);
        assert_eq!(testing_type_instance.as_integer(), Some(i128::from(integer)));
        assert_eq!(testing_type_instance.has_attribute("attribute"), false);
        assert_eq!(testing_type_instance.is_array(), false);
        assert_eq!(testing_type_instance.is_boolean(), false);
        assert_eq!(testing_type_instance.is_integer(), true);
        assert_eq!(testing_type_instance.is_null(), false);
        assert_eq!(testing_type_instance.is_number(), true);
        assert_eq!(testing_type_instance.is_object(), false);
        assert_eq!(testing_type_instance.is_string(), false);
    }

    #[test]
    fn test_testing_type_instance_list() {
        let array = vec![TestingType::from(1), TestingType::from(2)];
        let testing_type_instance = TestingType::from(array.clone());
        assert_eq!(testing_type_instance.as_array(), Some(array.iter().collect()));
        assert_eq!(testing_type_instance.has_attribute("attribute"), false);
        assert_eq!(testing_type_instance.is_array(), true);
        assert_eq!(testing_type_instance.is_boolean(), false);
        assert_eq!(testing_type_instance.is_integer(), false);
        assert_eq!(testing_type_instance.is_null(), false);
        assert_eq!(testing_type_instance.is_number(), false);
        assert_eq!(testing_type_instance.is_object(), false);
        assert_eq!(testing_type_instance.is_string(), false);
    }

    #[test]
    fn test_testing_type_instance_object() {
        let object: HashMap<String, TestingType> = [("attribute".to_string(), TestingType::from("value"))].iter().cloned().collect();
        let testing_type_instance = TestingType::from(object);
        assert_eq!(testing_type_instance.as_object().unwrap().items(), Some(vec![("attribute", &TestingType::from("value"))]));
        assert_eq!(testing_type_instance.get("attribute"), Some(&TestingType::from("value")));
        assert_eq!(testing_type_instance.has_attribute("attribute"), true);
        assert_eq!(testing_type_instance.is_array(), false);
        assert_eq!(testing_type_instance.is_boolean(), false);
        assert_eq!(testing_type_instance.is_integer(), false);
        assert_eq!(testing_type_instance.is_null(), false);
        assert_eq!(testing_type_instance.is_number(), false);
        assert_eq!(testing_type_instance.is_object(), true);
        assert_eq!(testing_type_instance.is_string(), false);
    }
}

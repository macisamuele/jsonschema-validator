maybe_import_dependencies_for_parallel_run!();

use crate::types::PrimitiveType;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Map<T>(T)
where
    T: PrimitiveType<T>;

impl<T> Deref for Map<T>
where
    T: PrimitiveType<T>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl<T> Map<T>
where
    T: PrimitiveType<T>,
{
    #[inline]
    pub fn new(value: &T) -> Self
    where
        Self: Sized,
    {
        Self(value.clone())
    }
}

pub trait Trait<T>
where
    T: PrimitiveType<T>,
{
    #[inline]
    fn keys(&self) -> Option<Vec<&str>>
    where
        T: 'static,
    {
        match self.items() {
            None => None,
            Some(items) => Some(iterate![items].map(|(key, _)| *key).collect()),
        }
    }

    #[inline]
    fn values(&self) -> Option<Vec<&T>> {
        match self.items() {
            None => None,
            Some(items) => Some(iterate![items].map(|(_, value)| *value).collect()),
        }
    }

    fn items(&self) -> Option<Vec<(&str, &T)>>;
}

#[cfg(test)]
mod json_map_trait_methods {
    use crate::testing_prelude::*;

    lazy_static! {
        static ref TESTING_MAP: TestingType = testing_map!["k1" => "v1", "k2" => "v2",];
        static ref TESTING_LIST: TestingType = testing_vec!["k1", "k2",];
    }

    #[test]
    fn keys_for_map() {
        let map = JsonMap::new(&TESTING_MAP.clone());
        let keys = map.keys();
        assert_eq!(
            keys.and_then(|mut vec| {
                vec.sort();
                Some(vec)
            }),
            Some(vec!["k1", "k2"])
        );
    }

    #[test]
    fn keys_for_non_map() {
        assert_eq!(JsonMap::new(&TESTING_LIST.clone()).keys(), None);
    }

    #[test]
    fn values_for_map() {
        let map = JsonMap::new(&TESTING_MAP.clone());
        let values = map.values();
        assert_eq!(
            values.and_then(|mut vec| {
                vec.sort_by_key(|testing_type| format!("{:?}", testing_type));
                Some(vec)
            }),
            Some(vec![&TestingType::from("v1"), &TestingType::from("v2")])
        );
    }

    #[test]
    fn values_for_non_map() {
        assert_eq!(JsonMap::new(&TESTING_LIST.clone()).values(), None);
    }

    #[test]
    fn items_for_map() {
        let map = JsonMap::new(&TESTING_MAP.clone());
        let values = map.items();
        assert_eq!(
            values.and_then(|mut vec| {
                vec.sort_by_key(|testing_type| format!("{:?}", testing_type));
                Some(vec)
            }),
            Some(vec![("k1", &TestingType::from("v1")), ("k2", &TestingType::from("v2"))])
        );
    }

    #[test]
    fn items_for_non_map() {
        assert_eq!(JsonMap::new(&TESTING_LIST.clone()).items(), None);
    }

    #[test]
    fn deref_does_its_job() {
        let primitive_value = TestingType::from("string");
        assert_eq!(*JsonMap::new(&primitive_value), primitive_value);
    }
}

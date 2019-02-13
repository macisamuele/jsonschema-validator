mod implementation;
mod stats;

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stats {
    hits: usize,
    misses: usize,
}

#[derive(Debug)]
struct Internal<K, V>
where
    K: Eq + Hash,
{
    data_holder: RwLock<HashMap<K, Arc<V>>>,
    stats: RwLock<Stats>, // RWLock needed to avoid requiring mutable reference to update stats
}

#[derive(Debug)]
pub struct Cache<K, V>(Internal<K, V>)
where
    K: Eq + Hash;

pub trait Cached<K, V>
where
    K: Eq + Hash,
{
    fn get(&self, key: &K) -> Option<Arc<V>>;

    fn set_from_arc(&self, key: K, value: Arc<V>);

    fn remove(&self, key: &K);

    fn clear(&self);

    fn size(&self) -> usize;

    fn hits(&self) -> usize;

    fn misses(&self) -> usize;

    fn stats(&self) -> Stats;

    fn set(&self, key: K, value: V) {
        self.set_from_arc(key, Arc::new(value))
    }

    fn get_or_fetch_with_result<E, F: Fn(&K) -> Result<V, E>>(&self, key: &K, fetcher: F) -> Result<Arc<V>, E>
    where
        K: Clone,
        V: Clone,
    {
        if let Some(t) = self.get(key) {
            Ok(t)
        } else {
            let value = Arc::new(fetcher(key)?);
            self.set_from_arc(key.clone(), Arc::clone(&value));
            Ok(value)
        }
    }

    fn get_or_fetch<F: Fn(&K) -> V>(&self, key: &K, fetcher: F) -> Arc<V>
    where
        K: Clone,
        V: Clone,
    {
        let result: Result<Arc<V>, ()> = self.get_or_fetch_with_result(key, |key_to_fetch| Ok(fetcher(key_to_fetch)));
        result.ok().unwrap()
    }
}

impl<K, V> Clone for Cache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone + PartialEq,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn default() -> Self {
        Self(Internal::default())
    }
}

impl<K, V> PartialEq for Cache<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V> Cached<K, V> for Cache<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn get(&self, key: &K) -> Option<Arc<V>> {
        self.0.get(key)
    }

    #[inline]
    fn set_from_arc(&self, key: K, value: Arc<V>) {
        self.0.set_from_arc(key, value)
    }

    #[inline]
    fn remove(&self, key: &K) {
        self.0.remove(key)
    }

    #[inline]
    fn clear(&self) {
        self.0.clear()
    }

    #[inline]
    fn size(&self) -> usize {
        self.0.size()
    }

    #[inline]
    fn hits(&self) -> usize {
        self.0.hits()
    }

    #[inline]
    fn misses(&self) -> usize {
        self.0.misses()
    }

    #[inline]
    fn stats(&self) -> Stats {
        self.0.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::Cache;
    use super::Stats;
    use crate::testing_prelude::*;
    use std::sync::Arc;

    #[test]
    fn test_cache_flows() {
        let cache: Cache<i32, i32> = Cache::default();
        assert_eq!(cache.get(&0), None);
        assert_eq!(cache.stats(), Stats { hits: 0, misses: 1 });
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.stats(), Stats { hits: 0, misses: 2 });

        cache.set(1, 2);
        cache.set(2, 0);

        assert_eq!(cache.get(&0), None);
        assert_eq!(cache.stats(), Stats { hits: 0, misses: 3 });
        assert_eq!(cache.get(&1), Some(Arc::new(2)));
        assert_eq!(cache.stats(), Stats { hits: 1, misses: 3 });
        assert_eq!(cache.get(&2), Some(Arc::new(0)));
        assert_eq!(cache.stats(), Stats { hits: 2, misses: 3 });

        assert_eq!(cache.get(&5), None);
        assert_eq!(cache.stats(), Stats { hits: 2, misses: 4 });
        assert_eq!(cache.get_or_fetch(&5, |k| k * k), Arc::new(25));
        assert_eq!(cache.stats(), Stats { hits: 2, misses: 5 });
        assert_eq!(cache.get_or_fetch(&5, |_| 0), Arc::new(25));
        assert_eq!(cache.stats(), Stats { hits: 3, misses: 5 });

        if let Err(val) = cache.get_or_fetch_with_result(&42, |&k| Err(k)) {
            assert_eq!(val, 42);
        } else {
            panic!("An error was expected during get_or_fetch_with_result call");
        }
    }
}

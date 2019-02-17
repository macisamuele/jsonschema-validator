use crate::cache::Cached;
use crate::cache::Internal;
use crate::cache::Stats;

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

impl<K, V> Clone for Internal<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data_holder: RwLock::new(self.data_holder.read().unwrap().clone()),
            stats: RwLock::new(*self.stats.read().unwrap()),
        }
    }
}

impl<K, V> Default for Internal<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self {
            data_holder: RwLock::new(HashMap::with_capacity(16)),
            stats: RwLock::new(Stats::default()),
        }
    }
}

impl<K, V> PartialEq for Internal<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // We consider two cache equivalent if they hold the same data, stats are not as important to be validated
        self.data_holder.read().unwrap().deref() == other.data_holder.read().unwrap().deref()
    }
}

#[allow(unsafe_code)]
unsafe impl<K, V> Send for Internal<K, V> where K: Eq + Hash {}

#[allow(unsafe_code)]
unsafe impl<K, V> Sync for Internal<K, V> where K: Eq + Hash {}

impl<K, V> Cached<K, V> for Internal<K, V>
where
    K: Eq + Hash,
{
    fn get(&self, key: &K) -> Option<Arc<V>> {
        self.data_holder
            .read()
            .unwrap()
            .get(key)
            .and_then(|value| {
                self.stats.write().unwrap().register_hit();
                Some(Arc::clone(value))
            })
            .or_else(|| {
                self.stats.write().unwrap().register_miss();
                None
            })
    }

    fn set_from_arc(&self, key: K, value: Arc<V>) {
        let _ = self.data_holder.write().unwrap().insert(key, value);
    }

    fn remove(&self, key: &K) {
        let _ = self.data_holder.write().unwrap().remove(key);
    }

    fn clear(&self) {
        self.data_holder.write().unwrap().clear();
        self.stats.write().unwrap().clear();
    }

    fn size(&self) -> usize {
        self.data_holder.read().unwrap().len()
    }

    fn hits(&self) -> usize {
        self.stats.read().unwrap().get_hits()
    }

    fn misses(&self) -> usize {
        self.stats.read().unwrap().get_misses()
    }

    fn stats(&self) -> Stats {
        *self.stats.read().unwrap()
    }
}

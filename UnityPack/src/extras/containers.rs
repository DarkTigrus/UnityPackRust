/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// A HashMap which remembers its insertion order.
pub struct OrderedMap<K: Hash + Eq, V> {
    items: HashMap<K, V>,
    indices: HashMap<usize, K>,
}

impl<K: Hash + Eq + fmt::Debug, V: fmt::Debug> fmt::Debug for OrderedMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut content = String::new();
        for i in 0..self.items.len() {
            let key = self.indices.get(&i).unwrap();
            content.push_str(&format!("{:?}: ", key));
            let v = self.items.get(&key).unwrap();
            if i == self.items.len() - 1 {
                content.push_str(&format!("{:?}", v));
            } else {
                content.push_str(&format!("{:?}, ", v));
            }
        }
        write!(f, "{{ {} }}", content)
    }
}

impl<K, V> Default for OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        OrderedMap {
            items: HashMap::new(),
            indices: HashMap::new(),
        }
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.items.contains_key(&k) {
            return self.items.insert(k, v);
        }

        let k_clone = k.clone();
        self.items.insert(k, v);
        self.indices.insert(self.items.len() - 1, k_clone);

        None
    }

    /// Returns the number of key-value pairs in this map
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.items.get(k)
    }

    pub fn keys(&self) -> Keys<K, V> {
        self.items.keys()
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let mut remove_idx: usize = 0;
        let mut idx_found = false;
        for (i, ik) in &self.indices {
            if ik == k {
                remove_idx = *i;
                idx_found = true;
                break;
            }
        }
        if idx_found {
            self.indices.remove(&remove_idx);
        }

        self.items.remove(k)
    }
}

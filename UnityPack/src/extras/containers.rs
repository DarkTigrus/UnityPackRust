/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use std::collections::HashMap;
use std::hash::Hash;

/// A HashMap which remembers its insertion order.
pub struct OrderedMap<K, V> {
    items: HashMap<K, V>,
    indices: HashMap<usize, K>,
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> OrderedMap<K, V> {
        OrderedMap {
            items: HashMap::new(),
            indices: HashMap::new(),
        }
    }

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

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.items.get(k)
    }
}

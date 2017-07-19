/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use std::collections::HashMap;
use std::hash::Hash;

/// A HashMap which remembers its insertion order. 
pub struct OrderedMap<'a, K: 'a, V> {
    items: HashMap<K, V>,
    indices: HashMap<usize, &'a K>,
}

impl<'a, K: 'a, V> OrderedMap<'a, K, V> where K: Eq + Hash {

    pub fn new() -> OrderedMap<'a, K, V> {
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
        let res = self.items.insert(k, v);

        match res {
            None => {
                self.indices.insert(self.items.len()-1, self.items.entry(k).key());
            },
            _ => {},
        };

        res
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
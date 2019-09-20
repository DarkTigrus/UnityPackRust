/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
pub mod containers;
pub mod lzma;

#[cfg(test)]
mod tests {

    use extras::containers::OrderedMap;

    #[test]
    fn test_ordered_map() {
        let mut map = OrderedMap::default();
        map.insert("D", 4);
        map.insert("E", 3);
        map.insert("S", 2);
        map.insert("K", 1);
        assert_eq!(*(map.get(&"S").unwrap()), 2);
        assert_eq!(*(map.get(&"E").unwrap()), 3);
        assert_eq!(*(map.get(&"K").unwrap()), 1);
        assert_eq!(*(map.get(&"D").unwrap()), 4);
    }
}

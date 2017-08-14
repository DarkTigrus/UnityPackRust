/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

macro_rules! tryGet {
    ($map: expr, $key: expr) => {
        match $map.get(&String::from($key)) {
            Some(item) => item,
            None => {
                return Err(Error::EngineError(format!("Item not found for key {}",$key)));
            }
        }
    };
}

pub mod texture;

use super::object::ObjectValue;
use extras::containers::OrderedMap;

pub struct EngineObject {
    map: OrderedMap<String, ObjectValue>,
}

pub enum EngineObjectVariant {
    EngineObject(EngineObject),
    NotImplemented(OrderedMap<String, ObjectValue>),
}

impl EngineObject {
    pub fn new(
        type_name: &String,
        ordered_map: OrderedMap<String, ObjectValue>,
    ) -> EngineObjectVariant {
        match type_name.as_ref() {
            "Texture2D" => EngineObjectVariant::EngineObject(EngineObject { map: ordered_map }),
            _ => EngineObjectVariant::NotImplemented(ordered_map),
        }
    }
}

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
                return Err(Error::EngineError(format!(
                    "Item not found for key {}",
                    $key
                )));
            }
        }
    };
}

macro_rules! tryConsume {
    ($map: expr, $key: expr) => {
        match $map.remove(&String::from($key)) {
            Some(item) => item,
            None => {
                return Err(Error::EngineError(format!(
                    "Item not found for key {}",
                    $key
                )));
            }
        }
    };
}

pub mod font;
pub mod mesh;
pub mod object;
pub mod text;
pub mod texture;

use super::object::ObjectValue;
use extras::containers::OrderedMap;

#[derive(Debug)]
pub struct EngineObject {
    pub map: OrderedMap<String, ObjectValue>,
}

pub enum EngineObjectVariant {
    EngineObject(EngineObject),
    NotImplemented(OrderedMap<String, ObjectValue>),
}

impl EngineObject {
    pub fn get_object(
        type_name: &str,
        ordered_map: OrderedMap<String, ObjectValue>,
    ) -> EngineObjectVariant {
        match type_name {
            // implemented engine object types
            "Texture2D" | "TextAsset" | "FontDef" | "Font" | "MonoBehaviour" | "AssetBundle"
            | "GameObject" | "Mesh" => {
                EngineObjectVariant::EngineObject(EngineObject { map: ordered_map })
            }
            _ => EngineObjectVariant::NotImplemented(ordered_map),
        }
    }
}

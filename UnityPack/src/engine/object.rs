/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use error::{Error, Result};
use extras::containers::OrderedMap;
use object::ObjectValue;
use super::EngineObject;

pub struct Object {
    pub name: String,
}

impl Object {
    pub fn new(map: &OrderedMap<String, ObjectValue>) -> Result<Object> {
        match map.get(&format!("m_Name")) {
            Some(val) => {
                let name = match val.to_string() {
                    Ok(s) => s,
                    Err(err) => {
                        return Err(err);
                    }
                };

                Ok(Object { name: name })
            }
            None => Ok(Object { name: format!("") }),
        }
    }
}

pub struct GameObject {
    pub object: Object,
    pub is_active: bool,
    pub component: Vec<ObjectValue>,
    pub layer: u32,
    pub tag: u16,
}

pub trait IntoGameObject {
    fn to_gameobject(self) -> Result<GameObject>;
}

impl IntoGameObject for EngineObject {
    fn to_gameobject(mut self) -> Result<GameObject> {
        Ok(GameObject {
            object: Object::new(&self.map)?,
            component: tryConsume!(self.map, "m_Component").into_vec()?,
            is_active: tryGet!(self.map, "m_IsActive").to_bool()?,
            layer: tryGet!(self.map, "m_Layer").to_u32()?,
            tag: tryGet!(self.map, "m_Tag").to_u16()?,
        })
    }
}

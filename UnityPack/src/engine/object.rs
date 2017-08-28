/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use error::Result;
use extras::containers::OrderedMap;
use object::ObjectValue;

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
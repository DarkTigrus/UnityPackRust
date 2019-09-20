/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use super::EngineObject;
use engine::object::Object;
use error::{Error, Result};

pub trait IntoTextAsset {
    fn to_textasset(self) -> Result<TextAsset>;
}

pub struct TextAsset {
    pub object: Object,
    pub path: String,
    pub script: String,
}

impl IntoTextAsset for EngineObject {
    fn to_textasset(self) -> Result<TextAsset> {
        Ok(TextAsset {
            object: Object::new(&self.map)?,
            path: tryGet!(self.map, "m_PathName").to_string()?,
            script: tryGet!(self.map, "m_Script").to_string()?,
        })
    }
}

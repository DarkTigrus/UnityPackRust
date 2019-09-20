/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use super::EngineObject;
use engine::object::Object;
use error::{Error, Result};
use std::os::unix::ffi::OsStringExt;

pub enum TextAssetScript {
    Plain(String),
    Binary(Vec<u8>),
}

pub trait IntoTextAsset {
    fn to_textasset(self) -> Result<TextAsset>;
}

pub struct TextAsset {
    pub object: Object,
    pub path: Option<String>,
    pub script: TextAssetScript,
}

impl IntoTextAsset for EngineObject {
    fn to_textasset(self) -> Result<TextAsset> {
        Ok(TextAsset {
            object: Object::new(&self.map)?,
            path: {
                let key = String::from("m_PathName");
                match self.map.get(&key) {
                    Some(item) => Some(item.to_string()?),
                    None => None,
                }
            },
            script: match tryGet!(self.map, "m_Script").to_osstring()?.into_string() {
                Ok(s) => TextAssetScript::Plain(s),
                Err(b) => TextAssetScript::Binary(b.into_vec()),
            },
        })
    }
}

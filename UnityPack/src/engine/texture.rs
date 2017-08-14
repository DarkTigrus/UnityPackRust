/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use super::EngineObject;

pub struct Texture2D {
    pub height: u32,
    pub width: u32,
    pub name: String,
}

pub trait IntoTexture2D {
    fn to_texture2d(&self) -> Texture2D;
}

impl IntoTexture2D for EngineObject {
    fn to_texture2d(&self) -> Texture2D {
        Texture2D{
            height: self.get("m_Height").unwrap().to_i32().unwrap() as u32,
            width: self.get("m_Width").unwrap().to_i32().unwrap() as u32,
            name: self.get("m_Name").unwrap().to_string().unwrap()
        }
    }
}

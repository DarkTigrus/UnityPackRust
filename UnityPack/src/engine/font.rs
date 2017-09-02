/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use error::{Error, Result};
use super::EngineObject;
use object::AssetPointer;
use object::ObjectValue::ObjectPointer;
use asset::Asset;

pub trait IntoFontAsset {
    fn to_fontasset(self, asset: &Asset) -> Result<FontAsset>;
}

pub struct FontAsset {
    pub line_space_modifier: f32,
    pub font_size_modifier: f32,
    pub font: AssetPointer,
    pub outline_modifier: f32,
    pub single_line_adjustment: f32,
    pub character_size_modifier: f32,
    pub unbound_character_size_modifier: f32,
}

impl IntoFontAsset for EngineObject {
    fn to_fontasset(self, asset: &Asset) -> Result<FontAsset> {
        Ok(FontAsset {
            line_space_modifier: tryGet!(self.map, "m_LineSpaceModifier".to_string()).to_f32()?,
            font_size_modifier: tryGet!(self.map, "m_FontSizeModifier").to_f32()?,
            font: {
                match tryGet!(self.map, "m_Font") {
                    &ObjectPointer(ref object_pointer) => AssetPointer {
                        file_name: asset.get_file_by_id(&object_pointer.file_id)?,
                        path_id: object_pointer.path_id,
                    },
                    _ => {
                        return Err(Error::EngineError(
                            format!("Value is not of ObjectPointer type"),
                        ));
                    }
                }
            },
            outline_modifier: tryGet!(self.map, "m_OutlineModifier").to_f32()?,
            single_line_adjustment: tryGet!(self.map, "m_SingleLineAdjustment").to_f32()?,
            character_size_modifier: tryGet!(self.map, "m_CharacterSizeModifier").to_f32()?,
            unbound_character_size_modifier: tryGet!(self.map, "m_UnboundCharacterSizeModifier")
                .to_f32()?,
        })
    }
}

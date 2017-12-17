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
use engine::object::Object;
use object::ToByteVec;

pub trait IntoFontDef {
    fn to_fontdef(self, asset: &Asset) -> Result<FontDef>;
}

pub struct FontDef {
    pub line_space_modifier: f32,
    pub font_size_modifier: f32,
    pub font: AssetPointer,
    pub outline_modifier: f32,
    pub single_line_adjustment: f32,
    pub character_size_modifier: f32,
    pub unbound_character_size_modifier: f32,
}

impl IntoFontDef for EngineObject {
    fn to_fontdef(self, asset: &Asset) -> Result<FontDef> {
        Ok(FontDef {
            line_space_modifier: tryGet!(self.map, "m_LineSpaceModifier").to_f32()?,
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

pub trait IntoFont {
    fn to_font(self) -> Result<Font>;
}

#[derive(Debug)]
pub struct Font {
    pub object: Object,
    pub ascent: f32,
    pub character_padding: i32,
    pub character_spacing: i32,
    pub font_size: f32,
    pub kerning: Option<f32>,
    pub line_spacing: f32,
    pub pixel_scale: f32,
    pub data: Vec<u8>,
}

impl IntoFont for EngineObject {
    fn to_font(self) -> Result<Font> {
        Ok(Font {
            object: Object::new(&self.map)?,
            ascent: tryGet!(self.map, "m_Ascent").to_f32()?,
            character_padding: tryGet!(self.map, "m_CharacterPadding").to_i32()?,
            character_spacing: tryGet!(self.map, "m_CharacterSpacing").to_i32()?,
            font_size: tryGet!(self.map, "m_FontSize").to_f32()?,
            kerning: {
                match self.map.get(&"m_Kerning".to_string()) {
                    Some(k) => Some(k.to_f32()?),
                    None => None,
                }
            },
            line_spacing: tryGet!(self.map, "m_LineSpacing").to_f32()?,
            pixel_scale: tryGet!(self.map, "m_PixelScale").to_f32()?,
            data: tryGet!(self.map, "m_FontData").to_byte_vec()?,
        })
    }
}

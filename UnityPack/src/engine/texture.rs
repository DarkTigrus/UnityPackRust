/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use super::EngineObject;
use object::ToByteVec;
use error::{Error, Result};

pub struct Texture2D {
    pub height: u32,
    pub width: u32,
    pub name: String,
    pub data: Vec<u8>,
    pub texture_format: TextureFormat,
}

impl IntoTexture2D for EngineObject {
    fn to_texture2d(self) -> Result<Texture2D> {
        Ok(Texture2D {
            height: tryGet!(self.map, "m_Height").to_i32()? as u32,
            width: tryGet!(self.map, "m_Width").to_i32()? as u32,
            name: tryGet!(self.map, "m_Name").to_string()?,
            data: tryGet!(self.map, "image data").to_byte_vec()?,
            texture_format: TextureFormat::from_u32(tryGet!(self.map, "m_TextureFormat").to_i32()? as u32)?,
        })
    }
}

pub trait IntoTexture2D {
    fn to_texture2d(self) -> Result<Texture2D>;
}

#[derive(Debug)]
pub enum TextureFormat {
    Alpha8,
    ARGB4444,
    RGB24,
    RGBA32,
    ARGB32,
    RGB565,

    // Direct3D
    DXT1,
    DXT5,

    RGBA4444,
    BGRA32,

    // Direct3D 10
    BC4,
    BC5,
    DXT1Crunched,
    DXT5Crunched,

    // Direct3D 11
    BC6H,

    // PowerVR
    PVRTC_RGB2,
    PVRTC_2BPP_RGB,
    PVRTC_RGBA2,
    PVRTC_2BPP_RGBA,
    PVRTC_RGB4,
    PVRTC_4BPP_RGB,
    PVRTC_RGBA4,
    PVRTC_4BPP_RGBA,

    // Ericsson (Android)
    ETC_RGB4,
    ATC_RGB4,
    ATC_RGBA8,

    // Adobe ATF
    ATF_RGB_DXT1,
    ATF_RGBA_JPG,
    ATF_RGB_JPG,

    // Ericsson
    EAC_R,
    EAC_R_SIGNED,
    EAC_RG,
    EAC_RG_SIGNED,
    ETC2_RGB,
    ETC2_RGBA1,
    ETC2_RGBA8,

    // OpenGL / GLES
    ASTC_RGB_4x4,
    ASTC_RGB_5x5,
    ASTC_RGB_6x6,
    ASTC_RGB_8x8,
    ASTC_RGB_10x10,
    ASTC_RGB_12x12,
    ASTC_RGBA_4x4,
    ASTC_RGBA_5x5,
    ASTC_RGBA_6x6,
    ASTC_RGBA_8x8,
    ASTC_RGBA_10x10,
    ASTC_RGBA_12x12,
}

impl TextureFormat {
    pub fn from_u32(n: u32) -> Result<Self> {
        match n {
            1 => Ok(TextureFormat::Alpha8),
            2 => Ok(TextureFormat::ARGB4444),
            3 => Ok(TextureFormat::RGB24),
            4 => Ok(TextureFormat::RGBA32),
            5 => Ok(TextureFormat::ARGB32),
            7 => Ok(TextureFormat::RGB565),

            // Direct3D
            10 => Ok(TextureFormat::DXT1),
            12 => Ok(TextureFormat::DXT5),

            13 => Ok(TextureFormat::RGBA4444),
            14 => Ok(TextureFormat::BGRA32),

            // Direct3D 10
            26 => Ok(TextureFormat::BC4),
            27 => Ok(TextureFormat::BC5),
            28 => Ok(TextureFormat::DXT1Crunched),
            29 => Ok(TextureFormat::DXT5Crunched),

            // Direct3D 11
            24 => Ok(TextureFormat::BC6H),

            // PowerVR
            30 => Ok(TextureFormat::PVRTC_RGB2), // PVRTC_2BPP_RGB
            31 => Ok(TextureFormat::PVRTC_RGBA2), // PVRTC_2BPP_RGBA
            32 => Ok(TextureFormat::PVRTC_RGB4), // PVRTC_4BPP_RGB
            33 => Ok(TextureFormat::PVRTC_RGBA4), // PVRTC_4BPP_RGBA

            // Ericsson (Android)
            34 => Ok(TextureFormat::ETC_RGB4),
            35 => Ok(TextureFormat::ATC_RGB4),
            36 => Ok(TextureFormat::ATC_RGBA8),

            // Adobe ATF
            38 => Ok(TextureFormat::ATF_RGB_DXT1),
            39 => Ok(TextureFormat::ATF_RGBA_JPG),
            40 => Ok(TextureFormat::ATF_RGB_JPG),

            // Ericsson
            41 => Ok(TextureFormat::EAC_R),
            42 => Ok(TextureFormat::EAC_R_SIGNED),
            43 => Ok(TextureFormat::EAC_RG),
            44 => Ok(TextureFormat::EAC_RG_SIGNED),
            45 => Ok(TextureFormat::ETC2_RGB),
            46 => Ok(TextureFormat::ETC2_RGBA1),
            47 => Ok(TextureFormat::ETC2_RGBA8),

            // OpenGL / GLES
            48 => Ok(TextureFormat::ASTC_RGB_4x4),
            49 => Ok(TextureFormat::ASTC_RGB_5x5),
            50 => Ok(TextureFormat::ASTC_RGB_6x6),
            51 => Ok(TextureFormat::ASTC_RGB_8x8),
            52 => Ok(TextureFormat::ASTC_RGB_10x10),
            53 => Ok(TextureFormat::ASTC_RGB_12x12),
            54 => Ok(TextureFormat::ASTC_RGBA_4x4),
            55 => Ok(TextureFormat::ASTC_RGBA_5x5),
            56 => Ok(TextureFormat::ASTC_RGBA_6x6),
            57 => Ok(TextureFormat::ASTC_RGBA_8x8),
            58 => Ok(TextureFormat::ASTC_RGBA_10x10),
            59 => Ok(TextureFormat::ASTC_RGBA_12x12),
            _ => Err(Error::EngineError(
                format!("Unidentified texture format: {}", n),
            )),
        }
    }
}

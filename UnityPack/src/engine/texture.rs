/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use super::EngineObject;
use object::ToByteVec;
use error::{Error, Result};
use bcndecode::{BcnDecoderFormat, BcnEncoding, decode};
use decrunch::CrunchedData;

pub struct Texture2D {
    pub height: u32,
    pub width: u32,
    pub name: String,
    pub data: Vec<u8>,
    pub texture_format: TextureFormat,
}

impl Texture2D {
    pub fn to_image(self) -> Result<Vec<u8>> {
        let encoding = match self.texture_format {
            TextureFormat::DXT1 | TextureFormat::DXT1Crunched => BcnEncoding::Bc1,
            TextureFormat::DXT5 | TextureFormat::DXT5Crunched => BcnEncoding::Bc3,
            TextureFormat::BC4 => BcnEncoding::Bc4,
            TextureFormat::BC5 => BcnEncoding::Bc5,
            TextureFormat::BC6H => BcnEncoding::Bc6H,
            // RAW formats
            TextureFormat::Alpha8 |
            TextureFormat::ARGB4444 |
            TextureFormat::RGBA4444 |
            TextureFormat::RGB565 |
            TextureFormat::RGB24 |
            TextureFormat::RGBA32 |
            TextureFormat::ARGB32 => {
                return Ok(self.data);
            }
            _ => {
                return Err(Error::EngineError(format!(
                    "Image encoding is not supported: {:?}",
                    self.texture_format
                )));
            }
        };

        let format = match self.texture_format.pixel_format() {
            PixelFormat::RGB |  PixelFormat::RGB16 => {
                BcnDecoderFormat::RGBA
            }
            _ => {
                match self.texture_format {
                    TextureFormat::BC4 => {
                        BcnDecoderFormat::LUM
                    }
                    TextureFormat::BC6H => {
                        BcnDecoderFormat::RGBA
                    }
                    _ => {
                        BcnDecoderFormat::RGBA
                    }
                }
            }
        };
        

        // decrunch if needed
        let input_data = match self.texture_format {
            TextureFormat::DXT1Crunched | TextureFormat::DXT5Crunched => {
                let crunched_data = CrunchedData::new(&self.data);
                match crunched_data.decode_level(0) {
                    Some(data) => data,
                    None => {return Err(Error::EngineError(format!(
                    "DXT decrunch failed"
                ))); } 
                }
            }
            _ => self.data
        };

        match decode(&input_data, self.width as usize, self.height as usize, encoding, format) {
            Ok(result) => Ok(result),
            Err(err) => Err(Error::from(err))
        }
    }
}

impl IntoTexture2D for EngineObject {
    fn to_texture2d(self) -> Result<Texture2D> {
        Ok(Texture2D {
            height: tryGet!(self.map, "m_Height").to_i32()? as u32,
            width: tryGet!(self.map, "m_Width").to_i32()? as u32,
            name: tryGet!(self.map, "m_Name").to_string()?,
            data: tryGet!(self.map, "image data").to_byte_vec()?,
            texture_format: TextureFormat::from_u32(
                tryGet!(self.map, "m_TextureFormat").to_i32()? as u32,
            )?,
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
    PvrtcRgb2,
    Pvrtc2bppRgb,
    PvrtcRgba2,
    Pvrtc2bppRgba,
    PvrtcRgb4,
    Pvrtc4bppRgb,
    PvrtcRgba4,
    Pvrtc4bppRgba,

    // Ericsson (Android)
    EtcRgb4,
    AtcRgb4,
    AtcRgba8,

    // Adobe ATF
    AtfRgbDxt1,
    AtfRgbaJpg,
    AtfRgbJpg,

    // Ericsson
    EacR,
    EacRSigned,
    EacRg,
    EacRgSigned,
    Etc2Rgb,
    Etc2Rgba1,
    Etc2Rgba8,

    // OpenGL / GLES
    AstcRgb4x4,
    AstcRgb5x5,
    AstcRgb6x6,
    AstcRgb8x8,
    AstcRgb10x10,
    AstcRgb12x12,
    AstcRgba4x4,
    AstcRgba5x5,
    AstcRgba6x6,
    AstcRgba8x8,
    AstcRgba10x10,
    AstcRgba12x12,
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
            30 => Ok(TextureFormat::PvrtcRgb2), // Pvrtc2bppRgb
            31 => Ok(TextureFormat::PvrtcRgba2), // Pvrtc2bppRgba
            32 => Ok(TextureFormat::PvrtcRgb4), // Pvrtc4bppRgb
            33 => Ok(TextureFormat::PvrtcRgba4), // Pvrtc4bppRgba

            // Ericsson (Android)
            34 => Ok(TextureFormat::EtcRgb4),
            35 => Ok(TextureFormat::AtcRgb4),
            36 => Ok(TextureFormat::AtcRgba8),

            // Adobe ATF
            38 => Ok(TextureFormat::AtfRgbDxt1),
            39 => Ok(TextureFormat::AtfRgbaJpg),
            40 => Ok(TextureFormat::AtfRgbJpg),

            // Ericsson
            41 => Ok(TextureFormat::EacR),
            42 => Ok(TextureFormat::EacRSigned),
            43 => Ok(TextureFormat::EacRg),
            44 => Ok(TextureFormat::EacRgSigned),
            45 => Ok(TextureFormat::Etc2Rgb),
            46 => Ok(TextureFormat::Etc2Rgba1),
            47 => Ok(TextureFormat::Etc2Rgba8),

            // OpenGL / GLES
            48 => Ok(TextureFormat::AstcRgb4x4),
            49 => Ok(TextureFormat::AstcRgb5x5),
            50 => Ok(TextureFormat::AstcRgb6x6),
            51 => Ok(TextureFormat::AstcRgb8x8),
            52 => Ok(TextureFormat::AstcRgb10x10),
            53 => Ok(TextureFormat::AstcRgb12x12),
            54 => Ok(TextureFormat::AstcRgba4x4),
            55 => Ok(TextureFormat::AstcRgba5x5),
            56 => Ok(TextureFormat::AstcRgba6x6),
            57 => Ok(TextureFormat::AstcRgba8x8),
            58 => Ok(TextureFormat::AstcRgba10x10),
            59 => Ok(TextureFormat::AstcRgba12x12),
            _ => Err(Error::EngineError(
                format!("Unidentified texture format: {}", n),
            )),
        }
    }

    pub fn pixel_format(&self) -> PixelFormat {
        match self {
            &TextureFormat::RGB24 => PixelFormat::RGB,
            &TextureFormat::ARGB32 => PixelFormat::ARGB,
            &TextureFormat::RGB565 => PixelFormat::RGB16,
            &TextureFormat::Alpha8 => PixelFormat::A,
            &TextureFormat::RGBA4444 => PixelFormat::RGBA4B,
            &TextureFormat::ARGB4444 => PixelFormat::ARGB4B,
            _ => PixelFormat::RGBA,
        }
    }
}

pub enum PixelFormat {
    RGB,
    RGBA,
    ARGB,
    RGB16,
    A,
    RGBA4B,
    ARGB4B,
}

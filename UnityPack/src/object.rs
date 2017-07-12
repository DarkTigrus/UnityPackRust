/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::Asset;
use std::io::{Read, Seek, SeekFrom, Error, Result, ErrorKind, BufReader, Cursor};
use binaryreader::{Teller, ReadExtras};
use std::fmt;
use resources;
use assetbundle::{AssetBundle, Signature};

pub struct ObjectInfo {
    pub type_id: i64,
    pub path_id: i64,
    pub class_id: i16,
    pub typename: String,
    data_offset: u32,
    size: u32,
    is_destroyed: bool,
}

impl ObjectInfo {

    pub fn new<R: Read+Seek+ Teller>(asset: &mut Asset, buffer: &mut R) -> Result<ObjectInfo> {
        
        let mut res = ObjectInfo{
            type_id: 0,
            path_id: 0,
            class_id: 0,
            typename: String::from("Unknown"),
            data_offset: 0,
            size: 0,
            is_destroyed: false,
        };

        res.path_id = try!(ObjectInfo::read_id(buffer, asset));
        res.data_offset = try!(buffer.read_u32(&asset.endianness)) + asset.data_offset;
        res.size = try!(buffer.read_u32(&asset.endianness));

        if asset.format < 17 {
            res.type_id = try!(buffer.read_i32(&asset.endianness)) as i64;
            res.class_id = try!(buffer.read_i16(&asset.endianness));
        } else {
            let type_id = try!(buffer.read_i32(&asset.endianness));
            let class_id = match &asset.tree {
                &Some(ref tree) => {
                    tree.class_ids[type_id as usize]
                },
                &None => {
                    return Err(Error::new(ErrorKind::InvalidData, "Asset's typemetadata is undefined"));
                },
            };
            res.type_id = class_id as i64;
            res.class_id = class_id as i16;
        }

        if asset.format <= 10 {
            res.is_destroyed = try!(buffer.read_i16(&asset.endianness)) != 0;
        } else if asset.format >= 11 && asset.format <= 16 {
            let _ = try!(buffer.read_i16(&asset.endianness)); // unknown

            if asset.format >= 15 {
                let _ = try!(buffer.read_u8()); // unknown
            }
        }

        return Ok(res);
    }

    fn read_id<R: Read+Seek+ Teller>(buffer: &mut R, asset: &mut Asset) -> Result<i64> {
        if asset.long_object_ids {
            return buffer.read_i64(&asset.endianness);
        }
        return asset.read_id(buffer);
    }

    pub fn get_type(&self, asset: &mut Asset, bundle: &mut AssetBundle) -> String {
        if self.type_id > 0 {
            return match resources::get_unity_class(&self.type_id) {
                Ok(type_str) => type_str,
                Err(_) => format!("<Unknown {}>", self.type_id),
            };
        } else if !asset.typenames.contains_key(&self.type_id) {
            let rawdata = self.read(asset, bundle);
        }
        String::new()
    }

    fn read(&self, asset: &mut Asset, bundle: &mut AssetBundle) -> Result<ObjectValue> {
        match bundle.signature {
            Signature::UnityFS(ref mut buf) => { return self.read_value(asset, buf); },
            Signature::UnityRaw(ref mut buf) => { return self.read_value(asset, buf); },
            Signature::UnityRawCompressed(ref mut buf) => { return self.read_value(asset, &mut BufReader::new(Cursor::new(buf.as_slice()))); },
            _ => { return Err( Error::new(ErrorKind::InvalidData, format!("Signature not supported for loading objects: {:?}", bundle.signature) )) },
        }
    }

    fn read_value<R: Read + Seek + Teller>(&self, asset: &mut Asset, buffer: &mut R) -> Result<ObjectValue> {
        let _ = buffer.seek(SeekFrom::Start(asset.bundle_offset as u64 + self.data_offset as u64));
        
        let mut object_buf = vec![0; self.size as usize];
        try!(buffer.read_exact(object_buf.as_mut_slice()));

        Ok(self.read_value_from_buffer(object_buf))
    }

    fn read_value_from_buffer(&self, buffer: Vec<u8>) -> ObjectValue {

        // TODO: read_value_from_buffer
        ObjectValue::None
    }
}

pub enum ObjectValue {
    Bool(bool),
    None,
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} {}>", self.typename, self.class_id)
    }
}

/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::Asset;
use std::io::{Read, Seek, Error, Result, ErrorKind};
use binaryreader::{Teller, ReadExtras};
use std::fmt;

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

    pub fn get_type(&self) -> String {
        String::new()
    }
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} {}>", self.typename, self.class_id)
    }
}

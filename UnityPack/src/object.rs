/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::Asset;
use std::io::{Read, Seek, Error, Result};
use binaryreader::Teller;

pub struct ObjectInfo {
    pub type_id: i64,
}

impl ObjectInfo {

    pub fn new<R: Read+Seek+ Teller>(asset: &mut Asset, buffer: &mut R) -> Result<ObjectInfo> {
        let res = ObjectInfo{
            type_id: 0,
        };
        
        
        let path_id = try!(res.read_id(buffer));
        
        
        
        
        return Ok(res);
    }

    fn read_id<R: Read+Seek+ Teller>(&mut self, buffer: &mut R) {

    }
}

/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::Asset;
use std::io::{Read, Seek, Error, Result};
use binaryreader::Teller;
use std::fmt;

pub struct ObjectInfo {
    pub type_id: i64,
    pub path_id: i64,
    pub class_id: i16,
    pub typename: String,
}

impl ObjectInfo {

    pub fn new<R: Read+Seek+ Teller>(asset: &mut Asset, buffer: &mut R) -> Result<ObjectInfo> {
        let res = ObjectInfo{
            type_id: 0,
            path_id: 0,
            class_id: 0,
            typename: String::from("Unknown"),
        };
        
        
        //let path_id = try!(res.read_id(buffer));
        
        // TODO
        
        
        return Ok(res);
    }

    fn read_id<R: Read+Seek+ Teller>(&mut self, buffer: &mut R) -> Result<i64> {
        // TODO
        Ok(0)
    }
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} {}>", self.typename, self.class_id)
    }
}

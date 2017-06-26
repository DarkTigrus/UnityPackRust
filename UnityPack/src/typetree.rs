/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use binaryreader::Teller;
use std::io::{Result, Read, Seek};
use binaryreader::ReadExtras;
use enums::{RuntimePlatform, get_runtime_platform};
use binaryreader::Endianness;

pub struct TypeMetadata {

}

impl TypeMetadata {

    pub fn new<R: Read+Seek+ Teller>(buffer: &mut R, format: u32, endianness: &Endianness) -> Result<TypeMetadata> {
        
        let generatorVersion = try!(buffer.read_string());
        let targetPlatform = get_runtime_platform(try!(buffer.read_u32(endianness)));

        if format >= 13 {
            let has_type_trees = try!(buffer.read_bool());
        }

        let result = TypeMetadata {};
        Ok(result)
    }
}
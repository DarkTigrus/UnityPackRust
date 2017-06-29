/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::Asset;
use std::io::{Read, Seek, Error};
use binaryreader::Teller;

pub struct ObjectInfo {}

impl ObjectInfo {

    pub fn new(asset: &mut Asset) -> ObjectInfo {
        let res = ObjectInfo{};
        return res;
    }

    pub fn load<R: Read+Seek+ Teller>(&mut self, buffer: &mut R) -> Option<Error> {
        None
    }
}

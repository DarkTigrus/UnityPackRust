/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use assetbundle::AssetBundle;
use assetbundle::Signature;
use assetbundle::FSDescriptor;
use binaryreader::Teller;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use byteorder::{BigEndian,ReadBytesExt};

pub struct Asset {
    pub name: String,
    bundle_offset: u64,
}

impl Asset {

    pub fn new(bundle: &mut AssetBundle) -> io::Result<Asset> {
        
        let is_compressed = bundle.is_compressed();
        let ref descriptor = bundle.descriptor;

        let mut buffer = match &mut bundle.signature {
            &mut Signature::UnityFS(ref mut buf) => {
                return Ok(Asset {
                    bundle_offset: buf.tell(),
                    name: String::new(),
                });
            },
            &mut Signature::UnityWeb(ref mut buf)|
            &mut Signature::UnityRaw(ref mut buf) => {
                buf
            },
            _ => {return Err(Error::new(ErrorKind::InvalidData, "Cannot load asset from unknown signature"));}
        };

        let mut asset = Asset {
            bundle_offset: buffer.tell(),
            name: String::new(),
        };      

        let header_size: u32;
        if !is_compressed {
            asset.name = try!(buffer.read_string());
			header_size = try!(buffer.read_u32::<BigEndian>());
			try!(buffer.read_u32::<BigEndian>());  // size
        } else {
            header_size = match descriptor {
                &FSDescriptor::Raw(ref desc) => {desc.asset_header_size},
                _ => {return Err(Error::new(ErrorKind::InvalidData, "Invalid raw descriptor"));},
            };
        }


        Ok(asset)
    }

}

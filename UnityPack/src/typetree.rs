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
use std::collections::HashMap;

pub struct TypeMetadata {
    generator_version: String,
    target_platform: RuntimePlatform,
    class_ids: Vec<i32>,
    hashes: HashMap<i32, Vec<u8>>,
    type_trees: HashMap<i32, TypeTree>,
}

pub struct TypeTree {

}

impl TypeTree {

    pub fn new(format: u32) -> Result<TypeTree> {
        let result = TypeTree {};
        
        Ok(result)
    }

    pub fn load<R: Read+Seek+ Teller>(&mut self, buffer: &mut R) {

    }
}

impl TypeMetadata {

    pub fn new<R: Read+Seek+ Teller>(buffer: &mut R, format: u32, endianness: &Endianness) -> Result<TypeMetadata> {
        
        let mut result = TypeMetadata {
            generator_version: String::new(),
            target_platform: RuntimePlatform::OSXEditor,
            class_ids: Vec::new(),
            hashes: HashMap::new(),
            type_trees: HashMap::new(),
        };

        result.generator_version = try!(buffer.read_string());
        result.target_platform = get_runtime_platform(try!(buffer.read_u32(endianness)));

        if format >= 13 {
            let has_type_trees = try!(buffer.read_bool());
            let num_types = try!(buffer.read_u32(endianness));

            for _ in 0..num_types {
                let mut class_id = try!(buffer.read_i32(endianness));
                if format >= 17 {
                    let _ = try!(buffer.read_u8()); // unk0
                    let script_id = try!(buffer.read_i16(endianness));
                    if class_id == 114 {
                        if script_id >= 0 {
                            //  make up a fake negative class_id to work like the
							// old system.  class_id of -1 is taken to mean that
							// the MonoBehaviour base class was serialized; that
							// shouldn't happen, but it's easy to account for.
                            class_id = -2 - (script_id as i32);
                        } else {
                            class_id = -1;
                        }
                    }
                }

                let hash_size;
                if class_id < 0 {
                    hash_size = 0x20;
                } else {
                    hash_size = 0x10;
                }
                let mut hash = vec![0; hash_size];
                try!(buffer.read_exact(hash.as_mut_slice()));

                result.class_ids.push(class_id);
                result.hashes.insert(class_id, hash);

                if has_type_trees {
                    let mut tree = try!(TypeTree::new(format));
                    tree.load(buffer);
                    result.type_trees.insert(class_id, tree);
                }
            }

        } else {
            let num_fields = try!(buffer.read_u32(endianness));
            for _ in 0..num_fields {
                let class_id = try!(buffer.read_i32(endianness));
                let mut tree = try!(TypeTree::new(format));
                tree.load(buffer);
                result.type_trees.insert(class_id, tree);
            }
        }

        Ok(result)
    }
}
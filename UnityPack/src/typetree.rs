/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use binaryreader::Teller;
use std::io::{Result, Read, Seek, BufReader, Cursor, Error, ErrorKind};
use binaryreader::{ReadExtras, BinaryReader};
use enums::{RuntimePlatform, get_runtime_platform};
use binaryreader::Endianness;
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::File;
use resources;
use std::error;

lazy_static! {
    static ref DEFAULT_TYPE_METADATA: Result<TypeMetadata> = {
        let file = try!(File::open(resources::RESOURCE_PATH_STRUCT));
        let mut bin_reader = BufReader::new(file);
        TypeMetadata::new(&mut bin_reader, 15, &Endianness::Big)
    };
}

pub fn default_type_metadata() -> Result<&'static TypeMetadata> {
    match DEFAULT_TYPE_METADATA.as_ref() {
        Ok(ref d) => Ok(d),
        Err(err) => {
            Err(Error::new(err.kind(), error::Error::description(err)))
        },
    }
}

pub struct TypeMetadata {
    generator_version: String,
    target_platform: RuntimePlatform,
    class_ids: Vec<i32>,
    hashes: HashMap<i32, Vec<u8>>,
    pub type_trees: HashMap<i64, Arc<TypeNode>>,
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
                    let tree = try!(TypeNode::new(format, buffer, &endianness));
                    result.type_trees.insert(class_id as i64, tree);
                }
            }

        } else {
            let num_fields = try!(buffer.read_u32(endianness));
            for _ in 0..num_fields {
                let class_id = try!(buffer.read_i32(endianness));
                let tree = try!(TypeNode::new(format, buffer, &endianness));
                result.type_trees.insert(class_id as i64, tree);
            }
        }

        Ok(result)
    }
}

pub struct TypeNode {
    type_name: String,
    field_name: String,
    size: i32,
    index: u32,
    is_array: bool,
    flags: i32,
    children: Vec<Arc<TypeNode>>,
}

impl TypeNode {

    pub fn new<R: Read+Seek+ Teller>(format: u32, buffer: &mut R, endianness: &Endianness) -> Result<Arc<TypeNode>> {
        if format == 10 || format >= 12 {
            return TypeNode::load_blob(buffer, endianness);
        } else {
            return TypeNode::load_old(buffer, endianness);
        };
    }

    fn load_blob<R: Read+Seek+ Teller>(buffer: &mut R, endianness: &Endianness) -> Result<Arc<TypeNode>> {
        // TODO:
        Err(Error::new(ErrorKind::InvalidData, "Not implemented yet"))
    }

    fn load_old<R: Read+Seek+ Teller>(buffer: &mut R, endianness: &Endianness) -> Result<Arc<TypeNode>> {

        let type_name = try!(buffer.read_string());
		let field_name = try!(buffer.read_string());
		let size = try!(buffer.read_i32(endianness));
		let index = try!(buffer.read_u32(endianness));
		let is_array = try!(buffer.read_i32(endianness)) == 1;
		let _ = try!(buffer.read_i32(endianness)); // version, unused
		let flags = try!(buffer.read_i32(endianness));

        let mut result = TypeNode {
            type_name: type_name,
            field_name: field_name,
            size: size,
            index: index,
            is_array: is_array,
            flags: flags,
            children: Vec::new(),
        };

        let num_fields = try!(buffer.read_u32(endianness));
        for _ in 0..num_fields {
            let tree = try!(TypeNode::load_old(buffer, endianness));
            result.children.push(tree);
        }

        Ok(Arc::new(result))
    }
}
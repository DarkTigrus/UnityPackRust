/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use binaryreader::Teller;
use std::io::{Result, Read, Seek, BufReader, Cursor, Error, ErrorKind};
use binaryreader::{ReadExtras, Endianness};
use enums::{RuntimePlatform, get_runtime_platform};
use std::collections::HashMap;
use std::sync::Arc;
use resources;

pub struct TypeMetadata {
    generator_version: String,
    target_platform: RuntimePlatform,
    pub class_ids: Vec<i32>,
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
                    result.type_trees.insert(class_id as i64, Arc::new(tree));
                }
            }

        } else {
            let num_fields = try!(buffer.read_u32(endianness));
            for _ in 0..num_fields {
                let class_id = try!(buffer.read_i32(endianness));
                let tree = try!(TypeNode::new(format, buffer, &endianness));
                result.type_trees.insert(class_id as i64, Arc::new(tree));
            }
        }

        Ok(result)
    }
}

pub struct TypeNode {
    pub type_name: String,
    pub field_name: String,
    pub size: i32,
    index: u32,
    is_array: bool,
    flags: i32,
    children: Vec<TypeNode>,
}

impl TypeNode {

    pub fn new<R: Read+Seek+ Teller>(format: u32, buffer: &mut R, endianness: &Endianness) -> Result<TypeNode> {
        if format == 10 || format >= 12 {
            return TypeNode::load_blob(buffer, endianness);
        } else {
            return TypeNode::load_old(buffer, endianness);
        };
    }

    fn load_blob<R: Read+Seek+ Teller>(buffer: &mut R, endianness: &Endianness) -> Result<TypeNode> {
        
        let num_nodes = try!(buffer.read_u32(endianness));
        let buffer_bytes = try!(buffer.read_u32(endianness));
       
        let mut node_data = vec![0; 24 * num_nodes as usize];
        try!(buffer.read_exact(node_data.as_mut_slice()));

        let mut stringbuffer_data = vec![0; buffer_bytes as usize];
        try!(buffer.read_exact(stringbuffer_data.as_mut_slice()));

        let mut buf = BufReader::new(Cursor::new(node_data.as_slice()));

        let mut parents: Vec<TypeNode> = Vec::new();

        let mut current_depth:i16 = -1;

        for _ in 0..num_nodes {
            // create root element
            let _ = try!(buf.read_i16(endianness)); // version, unused
            let depth = try!(buf.read_u8());

            let is_array = try!(buf.read_u8()) == 1;
            let type_name = try!(TypeNode::get_string_from_buffer(&buffer_bytes, &stringbuffer_data, &(try!(buf.read_i32(endianness))) ) );
            let field_name = try!(TypeNode::get_string_from_buffer(&buffer_bytes, &stringbuffer_data, &(try!(buf.read_i32(endianness))) ) );
            let size = try!(buf.read_i32(endianness));
            let index = try!(buf.read_u32(endianness));
            let flags = try!(buf.read_i32(endianness));

            let node = TypeNode {
                type_name: type_name,
                field_name: field_name,
                size: size,
                index: index,
                is_array: is_array,
                flags: flags,
                children: Vec::new(),
            };

            if depth as i16 > current_depth {
                parents.push(node);
                current_depth = depth as i16;
                continue;
            }

            // find parent of current node
            for _ in 0..((current_depth - depth as i16)+1) {
                let count = parents.len();
                let lastnode = parents.remove(count-1);
                parents.last_mut().unwrap().children.push(lastnode);
            }
            parents.push(node);
            current_depth = depth as i16;
        }

        // unwrap remaining nodes
        let elems = parents.len();
        for _ in 0..elems -1 {
            // remove last element and add it to the new last element as child
            let count = parents.len();
            let lastnode = parents.remove(count-1);
            parents.last_mut().unwrap().children.push(lastnode);
        }

        if parents.len() != 1 {
            return Err(Error::new(ErrorKind::InvalidData, "Failed to parse typetree"));
        }

        let root = parents.remove(0);
        Ok(root)
    }

    fn load_old<R: Read+Seek+ Teller>(buffer: &mut R, endianness: &Endianness) -> Result<TypeNode> {
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

        Ok(result)
    }

    fn get_string_from_buffer(buffer_bytes: &u32, buffer: &Vec<u8>, offset: &i32) -> Result<String> {
        let string_data: &Vec<u8>;
        let mut off: usize = *offset as usize;

        if *offset < 0 {
            off &= 0x7fffffff;
            string_data = try!(resources::default_type_strings());
        } else if *offset < *buffer_bytes as i32 {
            string_data = buffer;
        } else {
            return Ok(String::new());
        }

        let (_, right) = string_data.split_at(off);
        let mut k = right.split(|b| *b == 0 as u8);
        let z = k.next().unwrap();
        let result = String::from_utf8(z.to_vec()).unwrap();
        return Ok(result);
    }
}
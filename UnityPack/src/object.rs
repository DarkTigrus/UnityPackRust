/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::Asset;
use std::io::{Read, Seek, SeekFrom, BufReader, Cursor};
use std::io;
use error::{Error, Result};
use binaryreader::{Teller, ReadExtras, BinaryReader};
use std::fmt;
use std::sync::Arc;
use typetree::{TypeNode, DEFAULT_TYPENODE};
use resources::{default_type_metadata, get_unity_class};
use assetbundle::{AssetBundle, Signature};
use extras::containers::OrderedMap;

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
    pub fn new<R: Read + Seek + Teller>(asset: &mut Asset, buffer: &mut R) -> Result<ObjectInfo> {

        let mut res = ObjectInfo {
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
                &Some(ref tree) => tree.class_ids[type_id as usize],
                &None => {
                    return Err(Error::AssetError(
                        "Asset's typemetadata is undefined".to_string(),
                    ));
                }
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

    fn read_id<R: Read + Seek + Teller>(buffer: &mut R, asset: &mut Asset) -> io::Result<i64> {
        if asset.long_object_ids {
            return buffer.read_i64(&asset.endianness);
        }
        return asset.read_id(buffer);
    }

    pub fn get_type(&self, asset: &mut Asset, bundle: &mut AssetBundle) -> String {
        if self.type_id > 0 {
            return match get_unity_class(&self.type_id) {
                Ok(type_str) => type_str,
                Err(_) => format!("<Unknown {}>", self.type_id),
            };
        } else if !asset.typenames.contains_key(&self.type_id) {
            let rawdata = self.read(asset, bundle);
            // TODO
        }
        String::new()
    }

    fn get_type_tree(&self, asset: &mut Asset) -> Arc<TypeNode> {
        if self.type_id < 0 {
            match asset.tree {
                Some(ref tree) => {
                    if tree.type_trees.contains_key(&self.type_id) {
                        return tree.type_trees[&self.type_id].clone();
                    }
                    if tree.type_trees.contains_key(&(self.class_id as i64)) {
                        return tree.type_trees[&(self.class_id as i64)].clone();
                    }
                    match default_type_metadata() {
                        Ok(ref data) => {
                            if data.type_trees.contains_key(&(self.class_id as i64)) {
                                return data.type_trees[&(self.class_id as i64)].clone();
                            }
                        }
                        Err(_) => {}
                    };
                }
                None => {}
            };

        }
        asset.types[&self.type_id].clone()
    }

    fn read(&self, asset: &mut Asset, bundle: &mut AssetBundle) -> Result<ObjectValue> {
        match bundle.signature {
            Signature::UnityFS(ref mut buf) => {
                return self.read_value(asset, buf);
            }
            Signature::UnityRaw(ref mut buf) => {
                return self.read_value(asset, buf);
            }
            Signature::UnityRawCompressed(ref mut buf) => {
                return self.read_value(asset, &mut BufReader::new(Cursor::new(buf.as_slice())));
            }
            _ => return Err(Error::InvalidSignatureError),
        }
    }

    fn read_value<R: Read + Seek + Teller>(
        &self,
        asset: &mut Asset,
        buffer: &mut R,
    ) -> Result<ObjectValue> {
        let _ = buffer.seek(SeekFrom::Start(
            asset.bundle_offset as u64 + self.data_offset as u64,
        ));

        let mut object_buf = vec![0; self.size as usize];
        try!(buffer.read_exact(object_buf.as_mut_slice()));

        let typetree = self.get_type_tree(asset);

        let reader = BufReader::new(Cursor::new(object_buf));
        let mut binreader = BinaryReader::new(reader, asset.endianness.clone());
        self.read_value_from_buffer(asset, &typetree, &mut binreader)
    }

    fn read_value_from_buffer<R: Read + Seek>(
        &self,
        asset: &mut Asset,
        typetree: &TypeNode,
        buffer: &mut BinaryReader<R>,
    ) -> Result<ObjectValue> {
        let mut align = false;
        let expected_size = typetree.size;
        let pos_before = buffer.tell();
        let ref t = typetree.type_name;

        let mut result = ObjectValue::None;
        if t == "bool" {
            result = ObjectValue::Bool(try!(buffer.read_bool()));
        } else if t == "UInt8" {
            result = ObjectValue::U8(try!(buffer.read_u8()));
        } else if t == "SInt8" {
            result = ObjectValue::I8(try!(buffer.read_i8()));
        } else if t == "UInt16" {
            result = ObjectValue::U16(try!(buffer.read_u16()));
        } else if t == "SInt16" {
            result = ObjectValue::I16(try!(buffer.read_i16()));
        } else if t == "UInt32" || t == "unsigned int" {
            result = ObjectValue::U32(try!(buffer.read_u32()));
        } else if t == "SInt32" || t == "int" {
            result = ObjectValue::I32(try!(buffer.read_i32()));
        } else if t == "UInt64" {
            result = ObjectValue::U64(try!(buffer.read_u64()));
        } else if t == "SInt64" {
            result = ObjectValue::I64(try!(buffer.read_i64()));
        } else if t == "float" {
            result = ObjectValue::Float(try!(buffer.read_f32()));
        } else if t == "string" {
            let size = try!(buffer.read_u32());
            result = ObjectValue::String(try!(buffer.read_string_sized(size as usize)));
        } else {

            let ref first_child: TypeNode;
            if typetree.is_array {
                first_child = typetree;
            } else {
                first_child = match typetree.children.len() {
                    x if x > 0 => &typetree.children[0],
                    _ => &DEFAULT_TYPENODE,
                };
            }

            if t.contains("PPtr<") {
                let mut object_pointer = ObjectPointer::new(&typetree.type_name);
                result = match object_pointer.load(asset, buffer) {
                    Ok(_) => ObjectValue::ObjectPointer(object_pointer),
                    _ => ObjectValue::None,
                };
            } else if first_child.is_array {
                align = first_child.post_align();
                let size = try!(buffer.read_i32());
                let ref array_type = first_child.children[1].type_name;
                if array_type == "char" || array_type == "UInt8" {
                    let mut data: Vec<u8> = vec![0; size as usize];
                    try!(buffer.read_exact(data.as_mut_slice()));
                    result = ObjectValue::U8Array(data);
                } else {
                    // we dont know the type
                    let mut array: Vec<ObjectValue> = Vec::with_capacity(size as usize);
                    for _ in 0..size {
                        let object_value =
                            try!(self.read_value_from_buffer(asset, typetree, buffer));
                        array.push(object_value);
                    }
                    result = ObjectValue::Array(array);
                }
            } else if t == "pair" {
                if typetree.children.len() != 2 {
                    return Err(Error::ObjectError(format!(
                        "Type pair needs exactly 2 elements not {}",
                        typetree.children.len()
                    )));
                }

                let first = try!(self.read_value_from_buffer(
                    asset,
                    &typetree.children[0],
                    buffer,
                ));
                let second = try!(self.read_value_from_buffer(
                    asset,
                    &typetree.children[1],
                    buffer,
                ));
                result = ObjectValue::Pair((Box::new(first), Box::new(second)));
            } else {
                let mut ordered_map: OrderedMap<String, ObjectValue> = OrderedMap::new();

                for type_child in &typetree.children {
                    let child = try!(self.read_value_from_buffer(asset, type_child, buffer));
                    ordered_map.insert(type_child.field_name.clone(), child);
                }

                // let result = load_object(typetree, ordered_map);
            }


        }

        // TODO: read_value_from_buffer
        Ok(result)
    }
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} {}>", self.typename, self.class_id)
    }
}

pub enum ObjectValue {
    Bool(bool),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    Float(f32),
    String(String),
    ObjectPointer(ObjectPointer),
    U8Array(Vec<u8>),
    Array(Vec<ObjectValue>),
    Pair((Box<ObjectValue>, Box<ObjectValue>)),
    // TODO
    None,
}

pub struct ObjectPointer {
    type_name: String,
    file_id: i32,
    path_id: i64,
}

impl ObjectPointer {
    fn new(name: &String) -> ObjectPointer {
        ObjectPointer {
            type_name: name.clone(),
            file_id: 0,
            path_id: 0,
        }
    }

    fn load<R: Read + Seek>(
        &mut self,
        asset: &mut Asset,
        buffer: &mut BinaryReader<R>,
    ) -> Result<()> {
        self.file_id = try!(buffer.read_i32());
        self.path_id = try!(asset.read_id(buffer));

        Ok(())
    }
}

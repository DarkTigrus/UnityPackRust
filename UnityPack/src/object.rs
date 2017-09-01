/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use asset::{Asset, AssetOrRef};
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::io;
use std::clone::Clone;
use error::{Error, Result};
use binaryreader::{BinaryReader, ReadExtras, Teller};
use std::fmt;
use std::sync::Arc;
use typetree::{TypeNode, DEFAULT_TYPENODE};
use resources::{default_type_metadata, get_unity_class};
use assetbundle::Signature;
use extras::containers::OrderedMap;
use engine::{EngineObject, EngineObjectVariant};

#[derive(Debug)]
pub struct ObjectInfo {
    pub type_id: i64,
    pub path_id: i64,
    pub class_id: i16,
    pub type_name: String,
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
            type_name: String::from("Unknown"),
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

    pub fn get_type<R: Read + Seek + Teller>(&self, asset: &mut Asset, buffer: &mut R) -> String {
        if self.type_id > 0 {
            return match get_unity_class(&self.type_id) {
                Ok(type_str) => type_str,
                Err(_) => format!("<Unknown {}>", self.type_id),
            };
        } else if !asset.typenames.contains_key(&self.type_id) {
            let rawdata = match self.read(asset, buffer) {
                Ok(object_value) => object_value,
                Err(_) => {
                    return format!("<Unknown {}>", self.type_id);
                }
            };
            let typename = match rawdata {
                ObjectValue::EngineObject(engine_object) => {
                    self.get_script(asset, buffer, &engine_object.map)
                }
                ObjectValue::Map(map) => self.get_script(asset, buffer, &map),
                _ => format!("<Unknown {}>", self.type_id),
            };
            asset.typenames.insert(self.type_id, typename);
        }
        asset.typenames.get(&self.type_id).unwrap().clone()
    }

    fn get_script<R: Read + Seek + Teller>(
        &self,
        asset: &mut Asset,
        buffer: &mut R,
        map: &OrderedMap<String, ObjectValue>,
    ) -> String {
        match map.get(&"m_Script".to_string()) {
            Some(script) => match script {
                &ObjectValue::None => match &asset.tree {
                    &Some(ref tree) => match tree.type_trees.get(&self.type_id) {
                        Some(t_type) => t_type.type_name.clone(),
                        None => format!("{}", self.type_id),
                    },
                    _ => format!("{}", self.type_id),
                },
                &ObjectValue::ObjectPointer(ref pointer) => {
                    let script_obj = match pointer.resolve(asset, buffer) {
                        Ok(script_obj) => script_obj,
                        Err(_) => {
                            return format!("<Unknown {}>", self.type_id);
                        }
                    };
                    match script_obj {
                        ObjectValue::Map(script_map) => {
                            match script_map.get(&"m_ClassName".to_string()) {
                                Some(class_name) => match class_name {
                                    &ObjectValue::String(ref s) => s.clone(),
                                    _ => format!("<Unknown {}>", self.type_id),
                                },
                                None => format!("<Unknown {}>", self.type_id),
                            }
                        }
                        _ => pointer.type_name.clone(),
                    }
                }
                _ => format!("<Unknown {}>", self.type_id),
            },
            _ => match &asset.tree {
                &Some(ref tree) => match tree.type_trees.get(&self.type_id) {
                    Some(t_type) => t_type.type_name.clone(),
                    None => format!("{}", self.type_id),
                },
                _ => format!("{}", self.type_id),
            },
        }
    }

    fn get_type_tree(&self, asset: &Asset) -> Arc<TypeNode> {
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
                        Ok(ref data) => if data.type_trees.contains_key(&(self.class_id as i64)) {
                            return data.type_trees[&(self.class_id as i64)].clone();
                        },
                        Err(_) => {}
                    };
                }
                None => {}
            };
        }
        asset.types[&self.type_id].clone()
    }

    fn read<R: Read + Seek + Teller>(&self, asset: &Asset, buffer: &mut R) -> Result<ObjectValue> {
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

    pub fn read_signature(&self, asset: &Asset, signature: &mut Signature) -> Result<ObjectValue> {
        match signature {
            &mut Signature::UnityFS(ref mut buf) => {
                return self.read(asset, buf);
            }
            &mut Signature::UnityRaw(ref mut buf) => {
                return self.read(asset, buf);
            }
            &mut Signature::UnityRawCompressed(ref mut buf) => {
                return self.read(asset, &mut BufReader::new(Cursor::new(buf.as_slice())));
            }
            _ => return Err(Error::InvalidSignatureError),
        }
    }

    fn read_value_from_buffer<R: Read + Seek>(
        &self,
        asset: &Asset,
        typetree: &TypeNode,
        buffer: &mut BinaryReader<R>,
    ) -> Result<ObjectValue> {
        let mut align = false;
        let expected_size = typetree.size;
        let pos_before = buffer.tell();
        let ref t = typetree.type_name;

        let result;
        if t == "bool" {
            result = ObjectValue::Bool(try!(buffer.read_bool()));
        } else if t == "UInt8" {
            result = ObjectValue::U8(try!(buffer.read_u8()));
        } else if t == "SInt8" {
            result = ObjectValue::I8(buffer.read_i8()?);
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
            align = typetree.children[0].post_align();
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
                    Ok(_) => if object_pointer.is_valid() {
                        ObjectValue::ObjectPointer(object_pointer)
                    } else {
                        ObjectValue::None
                    },
                    _ => ObjectValue::None,
                };
            } else if first_child.is_array {
                align = first_child.post_align();
                let size = try!(buffer.read_u32());
                let ref array_type = first_child.children[1].type_name;
                if array_type == "char" || array_type == "UInt8" {
                    let mut data: Vec<u8> = vec![0; size as usize];
                    try!(buffer.read_exact(data.as_mut_slice()));
                    result = ObjectValue::U8Array(data);
                } else {
                    // we dont know the type
                    let mut array: Vec<ObjectValue> = Vec::with_capacity(size as usize);
                    for _ in 0..size {
                        let object_value = try!(self.read_value_from_buffer(
                            asset,
                            &first_child.children[1],
                            buffer,
                        ));
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

                let first = self.read_value_from_buffer(asset, &typetree.children[0], buffer)?;
                let second = self.read_value_from_buffer(asset, &typetree.children[1], buffer)?;
                result = ObjectValue::Pair((Box::new(first), Box::new(second)));
            } else {
                let mut ordered_map: OrderedMap<String, ObjectValue> = OrderedMap::new();

                for type_child in &typetree.children {
                    let child = try!(self.read_value_from_buffer(asset, type_child, buffer));
                    ordered_map.insert(type_child.field_name.clone(), child);
                }

                result = load_object(t, ordered_map);
                if t == "StreamedResource" {
                    //TODO: result.asset = self.resolve_streaming_asset(result.source);
                } else if t == "StreamingInfo" {
                    //TODO: result.asset = self.resolve_streaming_asset(result.path);
                }
            }
        }

        // Check to make sure we read at least as many bytes the tree says.
        // We allow reading more for the case of alignment.
        let pos_after = buffer.tell();
        let actual_size = pos_after - pos_before;
        if (expected_size > 0) && (actual_size < expected_size as u64) {
            return Err(Error::ObjectError(format!(
                "Expected read_value({}) to read {} bytes, but only read {} bytes",
                typetree,
                expected_size,
                actual_size
            )));
        }
        if align || typetree.post_align() {
            buffer.align();
        }

        Ok(result)
    }
}

fn load_object(type_name: &String, ordered_map: OrderedMap<String, ObjectValue>) -> ObjectValue {
    match EngineObject::new(type_name, ordered_map) {
        EngineObjectVariant::EngineObject(engine_object) => {
            ObjectValue::EngineObject(engine_object)
        }
        EngineObjectVariant::NotImplemented(map_object) => ObjectValue::Map(map_object),
    }
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} {}>", self.type_name, self.class_id)
    }
}

#[derive(Debug)]
pub enum ObjectValue {
    // Primitive types
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
    Map(OrderedMap<String, ObjectValue>),
    EngineObject(EngineObject),
    None,
}

pub trait ToByteVec<T> {
    fn to_byte_vec(&self) -> Result<Vec<T>>;
}

impl ObjectValue {
    pub fn to_bool(&self) -> Result<bool> {
        match self {
            &ObjectValue::Bool(b) => Ok(b),
            _ => Err(Error::ObjectError(
                "ObjectValue is not bool variant".to_string(),
            )),
        }
    }

    pub fn to_i32(&self) -> Result<i32> {
        match self {
            &ObjectValue::I32(b) => Ok(b),
            _ => Err(Error::ObjectError(
                "ObjectValue is not i32 variant".to_string(),
            )),
        }
    }

    pub fn to_f32(&self) -> Result<f32> {
        match self {
            &ObjectValue::Float(b) => Ok(b),
            _ => Err(Error::ObjectError(
                "ObjectValue is not f32 variant".to_string(),
            )),
        }
    }

    pub fn to_string(&self) -> Result<String> {
        match self {
            &ObjectValue::String(ref s) => Ok(s.clone()),
            _ => Err(Error::ObjectError(
                "ObjectValue is not string variant".to_string(),
            )),
        }
    }
}

impl ToByteVec<u8> for ObjectValue {
    fn to_byte_vec(&self) -> Result<Vec<u8>> {
        match self {
            &ObjectValue::U8Array(ref s) => Ok(s.clone()),
            _ => Err(Error::ObjectError(
                "ObjectValue is not u8 array variant".to_string(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct ObjectPointer {
    pub type_name: String,
    pub file_id: i32,
    pub path_id: i64,
}

impl ObjectPointer {
    fn new(name: &String) -> ObjectPointer {
        ObjectPointer {
            type_name: name.clone(),
            file_id: 0,
            path_id: 0,
        }
    }

    fn load<R: Read + Seek>(&mut self, asset: &Asset, buffer: &mut BinaryReader<R>) -> Result<()> {
        self.file_id = try!(buffer.read_i32());
        self.path_id = try!(asset.read_id(buffer));

        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.file_id != 0 || self.path_id != 0
    }

    fn resolve<R: Read + Seek + Teller>(
        &self,
        asset: &Asset,
        buffer: &mut R,
    ) -> Result<ObjectValue> {
        let res_asset = match asset.asset_refs[self.file_id as usize] {
            AssetOrRef::AssetRef(_) => {
                //asset_ref.resolve()
                // asset references are not yet implemented
                return Ok(ObjectValue::None);
            }
            AssetOrRef::Asset => asset,
        };
        res_asset.objects[&self.path_id].read(asset, buffer)
    }
}

#[derive(Debug)]
pub struct AssetPointer {
    pub file_name: String,
    pub path_id: i64,
}

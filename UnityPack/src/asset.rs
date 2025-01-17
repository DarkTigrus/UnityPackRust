/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use assetbundle::AssetBundle;
use assetbundle::FSDescriptor;
use assetbundle::Signature;
use binaryreader::*;
use error::{Error, Result};
use lzma;
use object::ObjectInfo;
use resources::default_type_metadata;
use std::collections::HashMap;
use std::io;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::sync::Arc;
use typetree::{TypeMetadata, TypeNode};
use uuid::Uuid;

pub struct Asset {
    pub name: String,
    pub bundle_offset: u64,
    pub objects: HashMap<i64, ObjectInfo>,
    pub is_loaded: bool,
    pub endianness: Endianness,
    pub tree: Option<TypeMetadata>,
    pub types: HashMap<i64, Arc<TypeNode>>,
    pub asset_refs: Vec<AssetOrRef>,
    adds: Vec<(i64, i32)>,
    pub typenames: HashMap<i64, String>,
    // properties
    metadata_size: u32,
    file_size: u32,
    pub format: u32,
    pub data_offset: u32,
    pub long_object_ids: bool,
}

impl Asset {
    pub fn new(bundle: &mut AssetBundle) -> Result<Asset> {
        let is_compressed = bundle.is_compressed();
        let descriptor = &bundle.descriptor;

        let decompressed: Vec<u8>;

        let mut asset = Asset {
            bundle_offset: 0,
            name: String::new(),
            objects: HashMap::new(),
            is_loaded: false,
            endianness: Endianness::Big,
            tree: None,
            types: HashMap::new(),
            // when requesting first element it should be the asset itself
            asset_refs: vec![AssetOrRef::Asset],
            adds: Vec::new(),
            typenames: HashMap::new(),
            metadata_size: 0,
            file_size: 0,
            format: 0,
            data_offset: 0,
            long_object_ids: false,
        };

        {
            let buffer = match &mut bundle.signature {
                Signature::UnityFS(ref mut buf) => {
                    asset.bundle_offset = buf.tell();
                    return Ok(asset);
                }
                &mut Signature::UnityWeb(ref mut buf) | &mut Signature::UnityRaw(ref mut buf) => {
                    buf
                }
                _ => {
                    return Err(Error::InvalidSignatureError);
                }
            };

            let offset = buffer.tell();

            let header_size: u32;
            if !is_compressed {
                asset.name = buffer.read_string()?;
                header_size = buffer.read_u32(Endianness::Big)?;
                buffer.read_u32(Endianness::Big)?; // size
            } else {
                header_size = match descriptor {
                    FSDescriptor::Raw(ref desc) => desc.asset_header_size,
                    _ => {
                        return Err(Error::AssetError("Invalid raw descriptor".to_string()));
                    }
                };
            }

            let ofs = buffer.tell(); // save current offset so pointer can be later restored
            if is_compressed {
                let mut compressed_data = Vec::new();
                try!(buffer.read_to_end(&mut compressed_data));
                decompressed = match lzma::decompress(&compressed_data) {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(Error::LZMADecompressionError(Box::new(err)));
                    }
                };
                asset.bundle_offset = 0;
                try!(buffer.seek(SeekFrom::Start(ofs))); // restore pointer
            } else {
                asset.bundle_offset = offset + u64::from(header_size) - 4;
                if asset.is_resource() {
                    asset.bundle_offset -= asset.name.len() as u64;
                }
                return Ok(asset);
            }
        }

        // replace buffer in signature
        bundle.signature = Signature::UnityRawCompressed(decompressed);

        Ok(asset)
    }

    pub fn is_resource(&self) -> bool {
        self.name.as_str().ends_with(".resource")
    }

    pub fn load_objects(&mut self, signature: &mut Signature) -> io::Result<()> {
        if !self.is_loaded {
            self.load(signature)?;
        }
        Ok(())
    }

    fn load(&mut self, signature: &mut Signature) -> Result<()> {
        if self.is_resource() {
            self.is_loaded = true;
            return Ok(());
        }

        match signature {
            Signature::UnityFS(ref mut buf) => {
                self.load_from_buffer(buf)?;
            }
            Signature::UnityRaw(ref mut buf) => {
                self.load_from_buffer(buf)?;
            }
            Signature::UnityRawCompressed(ref mut buf) => {
                self.load_from_buffer(&mut BufReader::new(Cursor::new(buf.as_slice())))?;
            }
            _ => {
                return Err(Error::AssetError(format!(
                    "Signature not supported for loading objects: {:?}",
                    signature
                )))
            }
        };

        Ok(())
    }

    fn load_from_buffer<R: Read + Seek + Teller>(&mut self, buffer: &mut R) -> Result<()> {
        let _ = buffer.seek(SeekFrom::Start(self.bundle_offset));
        self.metadata_size = buffer.read_u32(self.endianness)?;
        self.file_size = buffer.read_u32(self.endianness)?;
        self.format = buffer.read_u32(self.endianness)?;
        self.data_offset = buffer.read_u32(self.endianness)?;

        if self.format >= 9 {
            self.endianness = match buffer.read_u32(self.endianness)? {
                0 => Endianness::Little,
                _ => Endianness::Big,
            };
        }

        let tree = TypeMetadata::new(buffer, self.format, self.endianness)?;
        self.tree = Some(tree);

        if (self.format >= 7) && (self.format <= 13) {
            self.long_object_ids = buffer.read_u32(self.endianness)? != 0
        }

        let num_objects = buffer.read_u32(self.endianness)?;

        for _ in 0..num_objects {
            if self.format >= 14 {
                buffer.align();
            }
            let obj = ObjectInfo::new(self, buffer)?;
            self.register_object(obj)?;
        }

        if self.format >= 11 {
            let num_adds = buffer.read_u32(self.endianness)?;
            for _ in 0..num_adds {
                if self.format >= 14 {
                    buffer.align();
                }
                let id = self.read_id(buffer)?;
                let add = buffer.read_i32(self.endianness)?;
                self.adds.push((id, add));
            }
        }

        let num_refs = buffer.read_u32(self.endianness)?;
        if self.format >= 6 {
            for _ in 0..num_refs {
                let asset_ref = AssetRef::new(buffer, self.endianness)?;
                self.asset_refs.push(AssetOrRef::AssetRef(asset_ref));
            }
        }

        let unk_string = buffer.read_string()?;

        if unk_string != "" {
            return Err(Error::AssetError(format!(
                "Error while loading Asset, ending string is not empty but {:?}",
                unk_string
            )));
        }

        // we need to clone the keys to avoid borrow-checker problems
        let mut keys: Vec<i64> = Vec::with_capacity(self.objects.keys().len());
        {
            let hashed_keys = self.objects.keys();
            for k in hashed_keys {
                keys.push(*k);
            }
        }
        for k in keys {
            let mut obj = self.objects.remove(&k).unwrap();
            let type_name = obj.get_type(self, buffer);
            obj.type_name = type_name;
            self.typenames.insert(obj.type_id, obj.type_name.clone());
            self.objects.insert(k, obj);
        }

        self.is_loaded = true;
        Ok(())
    }

    fn register_object(&mut self, obj: ObjectInfo) -> Result<()> {
        let tree = match self.tree {
            Some(ref t) => t,
            None => return Ok(()),
        };

        match tree.type_trees.get(&obj.type_id) {
            Some(o_type) => {
                self.types.insert(obj.type_id, o_type.clone());
            }
            None => {
                match self.types.get(&obj.type_id) {
                    Some(_) => {}
                    None => {
                        let trees = &default_type_metadata()?.type_trees;
                        match trees.get(&(obj.class_id.into())) {
                            Some(o) => {
                                self.types.insert(obj.type_id, o.clone());
                            }
                            None => {
                                // log warning
                                println!("Warning: {:?} is absent from structs.dat", obj.class_id);
                                // self.types.insert(obj.type_id, None)
                            }
                        };
                    }
                };
            }
        };

        if self.objects.get(&obj.path_id).is_some() {
            return Err(Error::AssetError(format!(
                "Duplicate asset object: {} (path_id={})",
                obj, obj.path_id
            )));
        };

        self.objects.insert(obj.path_id, obj);
        Ok(())
    }

    pub fn read_id<R: Read + Seek + Teller>(&self, buffer: &mut R) -> io::Result<i64> {
        if self.format >= 14 {
            return buffer.read_i64(self.endianness);
        }
        let result = buffer.read_i32(self.endianness)?.into();
        Ok(result)
    }

    pub fn get_file_by_id(&self, id: i32) -> Result<String> {
        match self.asset_refs[id as usize] {
            AssetOrRef::Asset => Ok(self.name.clone()),
            AssetOrRef::AssetRef(ref a_ref) => Ok(a_ref.file_path.clone()),
        }
    }
}

#[allow(dead_code)]
pub struct AssetRef {
    asset_path: String,
    guid: Uuid,
    asset_type: i32,
    pub file_path: String,
    // probably want to add a reference to the calling Asset itself
}

impl AssetRef {
    pub fn new<R: Read + Seek + Teller>(
        buffer: &mut R,
        endianness: Endianness,
    ) -> Result<AssetRef> {
        let asset_path = buffer.read_string()?;
        let mut uuid_buffer = [0; 16];
        buffer.read_exact(&mut uuid_buffer)?;
        let guid = match Uuid::from_bytes(&uuid_buffer) {
            Ok(uuid) => uuid,
            Err(err) => return Err(Error::UuidError(format!("{}", err))),
        };
        let asset_type = buffer.read_i32(endianness)?;
        let file_path = buffer.read_string()?;

        Ok(AssetRef {
            asset_path,
            guid,
            asset_type,
            file_path,
        })
    }
}

pub enum AssetOrRef {
    Asset,
    AssetRef(AssetRef),
}

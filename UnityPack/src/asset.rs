/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use assetbundle::AssetBundle;
use assetbundle::Signature;
use assetbundle::FSDescriptor;
use typetree::{TypeMetadata, TypeTree, default_type_meta_data};
use binaryreader::*;
use object::ObjectInfo;
use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Result, BufReader, Read, Seek, SeekFrom, Error, ErrorKind};
use lzma;
use std::rc::Rc;

pub struct Asset {
    pub name: String,
    bundle_offset: u64,
    objects: HashMap<u64,ObjectInfo>,
    is_loaded: bool,
    endianness: Endianness,
    tree: Option<TypeMetadata>,
    types: HashMap<i64, Rc<TypeTree>>,
    asset_refs: Vec<AssetOrRef>,
    // properties
    metadata_size: u32,
    file_size: u32,
    format: u32,
    data_offset: u32,
    long_object_ids: bool,
}

impl Asset {
    pub fn new(bundle: &mut AssetBundle) -> Result<Asset> {

        let is_compressed = bundle.is_compressed();
        let ref descriptor = bundle.descriptor;

        let decompressed: Vec<u8>;

        let mut asset = Asset {
            bundle_offset: 0,
            name: String::new(),
            objects: HashMap::new(),
            is_loaded: false,
            endianness: Endianness::Big,
            tree: None,
            types: HashMap::new(),
            /// when requesting frist element it should be asset itself
            asset_refs: Vec::new(),
            metadata_size: 0,
            file_size: 0,
            format: 0,
            data_offset: 0,
            long_object_ids: false,
        };

        {
            let mut buffer = match &mut bundle.signature {
                &mut Signature::UnityFS(ref mut buf) => {
                    asset.bundle_offset = buf.tell();
                    return Ok(asset);
                }
                &mut Signature::UnityWeb(ref mut buf) |
                &mut Signature::UnityRaw(ref mut buf) => buf,
                _ => {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Cannot load asset from unknown signature"));
                }
            };

            let offset = buffer.tell();

            let header_size: u32;
            if !is_compressed {
                asset.name = try!(buffer.read_string());
                header_size = try!(buffer.read_u32(&Endianness::Big));
                try!(buffer.read_u32(&Endianness::Big)); // size
            } else {
                header_size = match descriptor {
                    &FSDescriptor::Raw(ref desc) => desc.asset_header_size,
                    _ => {
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid raw descriptor"));
                    }
                };
            }

            let ofs = buffer.tell(); // save current offset so pointer can be later restored
            if is_compressed {
                let mut compressed_data = Vec::new();
                try!(buffer.read_to_end(&mut compressed_data));
                decompressed = match lzma::decompress(&mut compressed_data) {
                    Ok(data) => data,
                    Err(err) => {
                        return Err(Error::new(ErrorKind::InvalidData, format!("{}", err)));
                    }
                };
                asset.bundle_offset = 0;
                try!(buffer.seek(SeekFrom::Start(ofs))); // restore pointer

            } else {
                asset.bundle_offset = offset + header_size as u64 - 4;
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

    pub fn get_objects(&mut self, bundle: &mut AssetBundle) -> io::Result<&HashMap<u64, ObjectInfo>> {
        if !self.is_loaded {
            isOptionError!(self.load(bundle));
        }
        Ok(&self.objects)
    }

    fn load(&mut self, bundle: &mut AssetBundle) -> Option<Error> {
        if self.is_resource() {
            self.is_loaded = true;
            return None;
        }

        match bundle.signature {
            Signature::UnityFS(ref mut buf) => { return self.load_from_buffer(buf); },
            Signature::UnityRaw(ref mut buf) => { return self.load_from_buffer(buf); },
            Signature::UnityRawCompressed(ref mut buf) => { return self.load_from_buffer(&mut BufReader::new(Cursor::new(buf.as_slice()))); },
            _ => { return Some( Error::new(ErrorKind::InvalidData, format!("Signature not supported for loading objects: {:?}", bundle.signature)  )) },
        }
    }

    fn load_from_buffer<R: Read+Seek+ Teller>(&mut self, buffer: &mut R) -> Option<Error> {
        let _ = buffer.seek(SeekFrom::Start(self.bundle_offset));

        self.metadata_size = tryOption!(buffer.read_u32(&self.endianness));
        self.file_size = tryOption!(buffer.read_u32(&self.endianness));
        self.format = tryOption!(buffer.read_u32(&self.endianness));
		self.data_offset = tryOption!(buffer.read_u32(&self.endianness));
        
        if self.format >= 9 {
            self.endianness = match tryOption!(buffer.read_u32(&self.endianness)) {
                0 => Endianness::Little,
                _ => Endianness::Big,
            };
        }

        let tree = tryOption!(TypeMetadata::new(buffer, self.format, &self.endianness));
        self.tree = Some(tree);
        
        if (self.format >= 7) && (self.format <= 13) {
            self.long_object_ids = tryOption!(buffer.read_u32(&self.endianness)) != 0
        }

        let num_objects = tryOption!(buffer.read_u32(&self.endianness));
        
        for _ in 0..num_objects {
            if self.format >= 14 {
                buffer.align();
            }
            let obj = tryOption!(ObjectInfo::new(self, buffer));
            tryOption!(self.register_object(obj));
        }

        if self.format >= 11 {
            let num_adds = tryOption!(buffer.read_u32(&self.endianness));
            for _ in 0..num_adds {
                if self.format >= 14 {
                    buffer.align();
                }
                let id = self.read_id(buffer);
                let add = tryOption!(buffer.read_i32(&self.endianness));
                self.adds.push((id, add));
            }
        }

        if self.format >= 6 {
            let num_refs = tryOption!(buffer.read_u32(&self.endianness));
            for _ in 0..num_refs {
                let asset_ref = AssetRef::new(buffer);
                self.asset_refs.push(asset_ref);
            }
        }
        
        let unk_string = tryOption!(buffer.read_string());
        
        if unk_string != "" {
            return Some(Error::new(ErrorKind::InvalidData, format!("Error while loading Asset, ending string is not empty but {:?}", unk_string)));
        }

        self.is_loaded = true;
        None
    }

    fn register_object(&mut self, obj: ObjectInfo) -> Option<Error> {
        let tree = match self.tree {
            Some(t) => t,
            None => return None,
        };
        match tree.type_trees.get(&obj.type_id) {
            Some(oType) => {self.types.insert(obj.type_id, oType.clone())},
            None => {
                match self.types.get(&obj.type_id) {
                    Some(_) => {},
                    None => {
                        let trees = tryOption!(default_type_meta_data).type_trees;
                        match trees.get(obj.class_id) {
                            Some(o) => self.types.insert(obj.type_id, o),
                            None => {
                                // log warning
                                println!("Warning: {} is absent from structs.dat", obj.class_id);
                                // self.types.insert(obj.type_id, None)
                            },
                        };
                    },
                }
            },
        };

        match self.objects.get(obj.path_id) {
            Some(_) => return Some(Error::new(ErrorKind::InvalidData, format!("Duplicate asset object: {} (path_id={})", obj, obj.path_id))),
            None => {},
        }
        None
    }

    fn read_id<R: Read+Seek+ Teller>(&mut self, buffer: &mut R) -> Result<i64> {
        if self.format >= 14 {
            return buffer.read_i64(&self.endianness);
        }
        let result = try!(buffer.read_i32(&self.endianness)) as i64;
        return Ok(result);
    }
}

struct AssetRef {
    asset_path: String,
}

impl AssetRef {
    pub fn new<R: Read+Seek+ Teller>(buffer: &mut R) -> Result<AssetRef> {
        let asset_path = try!(buffer.read_string());
        
        
        Ok(AssetRef {
            asset_path: asset_path,
        })
    }
}

enum AssetOrRef {
    Asset(Asset),
    AssetRef(AssetRef),
}

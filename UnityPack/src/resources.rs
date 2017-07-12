/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use typetree::TypeMetadata;
use std::io::{Read, Result, BufReader, ErrorKind};
use std::io;
use std::fs::File;
use binaryreader::{Endianness};
use std::error;
use std::collections::HashMap;
use serde_json;
use std::error::Error;

const RESOURCE_PATH_STRUCT: &str = "res/structs.dat";
const RESOURCE_PATH_STRINGS: &str = "res/strings.dat";
const RESOURCE_PATH_CLASSES: &str = "res/classes.json";

lazy_static! {
    static ref DEFAULT_TYPE_METADATA: Result<TypeMetadata> = {
        let file = try!(File::open(RESOURCE_PATH_STRUCT));
        let mut bin_reader = BufReader::new(file);
        TypeMetadata::new(&mut bin_reader, 15, &Endianness::Big)
    };
}

lazy_static! {
    static ref DEFAULT_TYPE_STRINGS: Result<Vec<u8>> = {
        let file = try!(File::open(RESOURCE_PATH_STRINGS));
        let mut bin_reader = BufReader::new(file);
        let mut result: Vec<u8> = Vec::new();
        let _ = bin_reader.read_to_end(&mut result);
        Ok(result)
    };
}

lazy_static! {
    static ref UNITY_CLASSES: Result<HashMap<i64, String>> = {
        let file = try!(File::open(RESOURCE_PATH_CLASSES));
        let bin_reader = BufReader::new(file);
        
        let json_object: serde_json::Value = match serde_json::from_reader(bin_reader) {
            Ok(obj) => obj,
            Err(err) => {
                println!("Failed to read {}", RESOURCE_PATH_STRUCT);
                return Err(io::Error::new(ErrorKind::InvalidData, err.description()));
            },
        };
        let object_map = json_object.as_object().unwrap();

        let mut result: HashMap<i64, String> = HashMap::new();
        for (k,v) in object_map {
            result.insert(k.parse().unwrap(), v.as_str().unwrap().to_string());
        }

        Ok(result)
    };
}

pub fn default_type_metadata() -> Result<&'static TypeMetadata> {
    match DEFAULT_TYPE_METADATA.as_ref() {
        Ok(ref d) => Ok(d),
        Err(err) => {
            println!("Failed to read {}", RESOURCE_PATH_STRUCT);
            Err(io::Error::new(err.kind(), error::Error::description(err)))
        },
    }
}

pub fn default_type_strings() -> Result<&'static Vec<u8>> {
    match DEFAULT_TYPE_STRINGS.as_ref() {
        Ok(ref d) => Ok(d),
        Err(err) => {
            println!("Failed to read {}", RESOURCE_PATH_STRINGS);
            Err(io::Error::new(err.kind(), error::Error::description(err)))
        },
    }
}

pub fn get_unity_class(type_id: &i64) -> Result<String> {
    match UNITY_CLASSES.as_ref() {
        Ok(ref m) => {
            Ok(m[type_id].clone())
            },
        Err(err) => {
            println!("Failed to read {}", RESOURCE_PATH_CLASSES);
            Err(io::Error::new(err.kind(), error::Error::description(err)))
        },
    }
}

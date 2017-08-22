/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use typetree::TypeMetadata;
use std::io::{BufReader, Cursor};
use std::io;
use error::{Error, Result};
use binaryreader::Endianness;
use std::collections::HashMap;
use serde_json;
use std;

lazy_static! {
    static ref DEFAULT_TYPE_METADATA: Result<TypeMetadata> = {
        let data: &[u8] = include_bytes!("../res/structs.dat");
        let mut bin_reader = BufReader::new(Cursor::new(data));
        TypeMetadata::new(&mut bin_reader, 15, &Endianness::Big)
    };
}

lazy_static! {
    static ref DEFAULT_TYPE_STRINGS: Result<Vec<u8>> = {
        let data: &[u8] = include_bytes!("../res/strings.dat");
        let result = Vec::from(data);
        Ok(result)
    };
}

lazy_static! {
    static ref UNITY_CLASSES: Result<HashMap<i64, String>> = {
        let data: &[u8] = include_bytes!("../res/classes.json");
        let json_object: serde_json::Value = match serde_json::from_slice(data) {
            Ok(obj) => obj,
            Err(err) => {
                eprintln!("Resource import error: Failed to read classes.json");
                return Err(Error::ResourceError(format!("{}",err)));
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
            eprintln!("Failed to read structs.dat");
            match err {
                &Error::IOError(ref e) => {
                    Err(Error::IOError(Box::new(
                        io::Error::new(e.kind(), std::error::Error::description(e)),
                    )))
                }
                &Error::ResourceError(ref s) => Err(Error::ResourceError(s.clone())),
                _ => Err(Error::ResourceError("Unknown".to_string())),
            }
        }
    }
}

pub fn default_type_strings() -> Result<&'static Vec<u8>> {
    match DEFAULT_TYPE_STRINGS.as_ref() {
        Ok(ref d) => Ok(d),
        Err(err) => {
            eprintln!("Failed to read strings.dat");
            match err {
                &Error::IOError(ref e) => {
                    Err(Error::IOError(Box::new(
                        io::Error::new(e.kind(), std::error::Error::description(e)),
                    )))
                }
                &Error::ResourceError(ref s) => Err(Error::ResourceError(s.clone())),
                _ => Err(Error::ResourceError("Unknown".to_string())),
            }
        }
    }
}

pub fn get_unity_class(type_id: &i64) -> Result<String> {
    match UNITY_CLASSES.as_ref() {
        Ok(ref m) => Ok(m[type_id].clone()),
        Err(err) => {
            eprintln!("Failed to read classes.json");
            match err {
                &Error::IOError(ref e) => {
                    Err(Error::IOError(Box::new(
                        io::Error::new(e.kind(), std::error::Error::description(e)),
                    )))
                }
                &Error::ResourceError(ref s) => Err(Error::ResourceError(s.clone())),
                _ => Err(Error::ResourceError("Unknown".to_string())),
            }
        }
    }
}

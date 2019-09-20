/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

use bcndecode;
use std::error;
use std::fmt;
use std::io;
use std::result;

#[derive(Debug)]
pub enum Error {
    LZ4DecompressionError(Box<dyn error::Error + Send + Sync>),
    LZMADecompressionError(Box<dyn error::Error + Send + Sync>),
    BCNDecodeError(Box<bcndecode::Error>),
    CompressionNotImplementedError,
    FeatureNotImplementedError,
    ValueNotFoundError,
    DataReadError,
    CustomError(String),
    InvalidSignatureError,
    IOError(Box<io::Error>),
    UuidError(String),
    AssetError(String),
    ObjectError(String),
    TypeError(String),
    ResourceError(String),
    EngineError(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::LZ4DecompressionError(ref err) => err.description(),
            Error::LZMADecompressionError(ref err) => err.description(),
            Error::BCNDecodeError(ref err) => err.description(),
            Error::CompressionNotImplementedError => {
                "Requested decompression method is not yet implemented"
            }
            Error::ValueNotFoundError => "Requested value was not found",
            Error::DataReadError => "Failed to read stream data",
            Error::FeatureNotImplementedError => "Requested feature is not yet implemented",
            Error::InvalidSignatureError => "Signature is invalid",
            Error::IOError(ref err) => err.description(),
            Error::UuidError(ref s) => s,
            Error::AssetError(ref s) => s,
            Error::ObjectError(ref s) => s,
            Error::TypeError(ref s) => s,
            Error::ResourceError(ref s) => s,
            Error::CustomError(ref s) => s,
            Error::EngineError(ref s) => s,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", error::Error::description(self))
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IOError(Box::new(error))
    }
}

impl From<bcndecode::Error> for Error {
    fn from(error: bcndecode::Error) -> Error {
        Error::BCNDecodeError(Box::new(error))
    }
}

pub type Result<T> = result::Result<T, Error>;

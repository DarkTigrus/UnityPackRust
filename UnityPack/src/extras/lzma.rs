/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use lzma_sys::*;
use error::{Error, Result};
use byteorder::{ReadBytesExt, LittleEndian};
use std::mem;
use libc;

pub fn decompress_raw(mut compressed_data: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
    // LZMA decompression: Unity does not provide the uncompressed size
    // so we have to insert it manually to get a proper lzma alone header
    let mut props = ReadBytesExt::read_u8(&mut compressed_data)?;
    let dict_size = ReadBytesExt::read_u32::<LittleEndian>(&mut compressed_data)?;
    let lc = props % 9;
    props /= 9;
    let pb = props / 5;
    let lp = props % 5;

    unsafe {

        let mut option: lzma_options_lzma = mem::zeroed();
        lzma_lzma_preset(&mut option as *mut lzma_options_lzma, LZMA_PRESET_DEFAULT);

        option.dict_size = dict_size;
        option.lc = lc as u32;
        option.lp = lp as u32;
        option.pb = pb as u32;

        let filters = vec![
            lzma_filter {
                id: LZMA_FILTER_LZMA1,
                options: &mut option as *mut _ as *mut libc::c_void,
            },
            lzma_filter {
                id: LZMA_VLI_UNKNOWN,
                options: &mut 0 as *mut _ as *mut libc::c_void,
            },
        ];

        let mut stream: lzma_stream = mem::zeroed();
        lzma_check(lzma_raw_decoder(
            &mut stream as *mut lzma_stream,
            filters.as_ptr(),
        ))?;

        // decode
        let mut output: Vec<u8> = vec![0; decompressed_size];
        stream.next_in = compressed_data.as_ptr();
        stream.avail_in = compressed_data.len();
        stream.next_out = output.as_mut_ptr();
        stream.avail_out = output.len();
        lzma_check(lzma_code(&mut stream, LZMA_RUN))?;

        Ok(output)
    }
}

fn lzma_check(ret: lzma_ret) -> Result<()> {
    match ret {
        LZMA_OK | LZMA_GET_CHECK | LZMA_NO_CHECK | LZMA_STREAM_END => Ok(()),
        LZMA_UNSUPPORTED_CHECK => Err(Error::LZMADecompressionError(Box::new(Error::CustomError(
            "Unsupported integrity check".to_string(),
        )))),
        LZMA_MEM_ERROR => Err(Error::LZMADecompressionError(
            Box::new(Error::CustomError("Memory error".to_string())),
        )),
        LZMA_MEMLIMIT_ERROR => Err(Error::LZMADecompressionError(Box::new(Error::CustomError(
            "Memory usage limit exceeded".to_string(),
        )))),
        LZMA_FORMAT_ERROR => Err(Error::LZMADecompressionError(Box::new(Error::CustomError(
            "Input format not supported by decoder".to_string(),
        )))),
        LZMA_OPTIONS_ERROR => Err(Error::LZMADecompressionError(Box::new(Error::CustomError(
            "Invalid or unsupported options".to_string(),
        )))),
        LZMA_DATA_ERROR => Err(Error::LZMADecompressionError(Box::new(
            Error::CustomError("Corrupt input data".to_string()),
        ))),
        LZMA_BUF_ERROR => Err(Error::LZMADecompressionError(Box::new(
            Error::CustomError("Insufficient buffer space".to_string()),
        ))),
        LZMA_PROG_ERROR => Err(Error::LZMADecompressionError(
            Box::new(Error::CustomError("Internal error".to_string())),
        )),
        _ => Err(Error::LZMADecompressionError(Box::new(Error::CustomError(
            format!("Unrecognized error from liblzma: {}", ret),
        )))),
    }
}

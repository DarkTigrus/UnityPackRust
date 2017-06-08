/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use std::cmp;
use std::io;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Cursor;
use asset::Asset;
use binaryreader::*;
use lz4_compress;

macro_rules! tryOption {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => return Some(err),
    });
}

macro_rules! isOptionError {
    ($e:expr) => (match $e {
        Some(err) => return Err(err),
		_ => {}
    });
}

macro_rules! tryVoid {
    ($e:expr) => (match $e {
        Err(err) => return Some(err),
		_ => {},
    });
}

fn decompress_data(data: &Vec<u8>, compression_type: &CompressionType) -> io::Result<Vec<u8>> {
    match *compression_type {
        CompressionType::LZ4 |
        CompressionType::LZ4HC => {
            println!("{:?}", data);
            match lz4_compress::decompress(data.as_slice()) {
                Err(err) => {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          format!("LZ4 decompression failed: {:?}", err)));
                }
                Ok(buf) => Ok(buf),
            }
        }
        CompressionType::LZMA | CompressionType::LZHAM => {
            Err(Error::new(ErrorKind::InvalidData,
                           format!("{:?} is not yet implemented", *compression_type)))
        }
        _ => Ok(data.clone()),
    }
}

custom_derive! {
    #[derive(Debug, EnumFromStr)]
    enum Signature {
        UnityFS,
        UnityWeb,
        UnityRaw,
        UnityArchive,
		Unknown,
    }
}

#[derive(Debug)]
struct UnityFsDescriptor {
    fs_file_size: i64,
    ci_block_size: u32,
    ui_block_size: u32,
}

#[derive(Debug)]
struct RawDescriptor {
    file_size: u32,
    header_size: i32,
    file_count: i32,
    bundle_count: i32,
    bundle_size: u32,
    uncompressed_bundle_size: u32,
    compressed_file_size: u32,
    asset_header_size: u32,
    num_assets: u32,
}

enum FSDescriptor {
    UnityFs(UnityFsDescriptor),
    Raw(RawDescriptor),
    Unknown,
}

#[derive(Debug, PartialEq)]
enum CompressionType {
    None,
    LZMA,
    LZ4,
    LZ4HC,
    LZHAM,
    Unknown,
}

impl CompressionType {
    fn from(x: &u8) -> CompressionType {
        match x {
            x if *x == CompressionType::None as u8 => CompressionType::None,
            x if *x == CompressionType::LZMA as u8 => CompressionType::LZMA,
            x if *x == CompressionType::LZ4 as u8 => CompressionType::LZ4,
            x if *x == CompressionType::LZ4HC as u8 => CompressionType::LZ4HC,
            x if *x == CompressionType::LZHAM as u8 => CompressionType::LZHAM,
            _ => CompressionType::Unknown,
        }
    }
}

/// An AssetBundle Object contains a lookup from path name to individual objects in the bundle.
pub struct AssetBundle {
    signature: Signature,
    format_version: u32,
    target_version: String, // also called as unity_version
    generator_version: String,
    descriptor: FSDescriptor,
    assets: Vec<Asset>,
}

impl Default for AssetBundle {
    fn default() -> AssetBundle {
        AssetBundle {
            signature: Signature::Unknown,
            format_version: 0,
            target_version: String::new(),
            generator_version: String::new(),
            descriptor: FSDescriptor::Unknown,
            assets: Vec::new(),
        }
    }
}

impl AssetBundle {
    pub fn load_from_file(file_path: &str) -> Result<AssetBundle, Error> {

        // open file
        let file = try!(File::open(file_path));
        let mut bin_reader = BinaryReader::new(BufReader::new(file), Endianness::Big);

        let mut result = AssetBundle {
            signature: Signature::Unknown,
            format_version: 0,
            target_version: String::new(),
            generator_version: String::new(),
            descriptor: FSDescriptor::Unknown,
            assets: Vec::new(),
        };

        // read header
        let signature_str = try!(bin_reader.read_string());
        if let Ok(x) = signature_str.parse() {
            result.signature = x;
        } else {
            result.signature = Signature::Unknown;
        }

        match result.signature {
            Signature::UnityArchive => {
                result.load_unityarchive();
            }
            Signature::UnityFS => {
                isOptionError!(result.load_unityfs(bin_reader));
            }
            Signature::UnityWeb | Signature::UnityRaw => {
                result.load_raw();
            }
            _ => {
                return Err(Error::new(ErrorKind::InvalidData,
                                      format!("Unknown format found: {}", signature_str)));
            }
        }

        Ok(result)
    }

    fn load_unityfs<R>(&mut self, mut buffer: BinaryReader<R>) -> Option<Error>
        where R: Read + Seek
    {
        self.format_version = tryOption!(buffer.read_u32());
        self.target_version = tryOption!(buffer.read_string());
        self.generator_version = tryOption!(buffer.read_string());

        let file_size = tryOption!(buffer.read_i64());
        let ciblock_size = tryOption!(buffer.read_u32());
        let uiblock_size = tryOption!(buffer.read_u32());

        self.descriptor = FSDescriptor::UnityFs(UnityFsDescriptor {
                                                    fs_file_size: file_size,
                                                    ci_block_size: ciblock_size,
                                                    ui_block_size: uiblock_size,
                                                });

        let flags = (tryOption!(buffer.read_u32()) as u8) & 0x3F;
        let compression_type = CompressionType::from(&flags);
        let raw_data = tryOption!(buffer.read_bytes((ciblock_size as usize)));

        let decompressed_data = tryOption!(decompress_data(&raw_data, &compression_type));
        let dreader = BufReader::new(Cursor::new(decompressed_data.as_slice()));
        let mut data_reader = BinaryReader::new(dreader, Endianness::Big);

        tryVoid!(data_reader.read_bytes(16)); // guid

        let num_blocks = tryOption!(data_reader.read_u32());
        let mut blocks: Vec<ArchiveBlockInfo> = vec![];

        for _ in 0..num_blocks {
            let bu_size = tryOption!(data_reader.read_u32());
            let bc_size = tryOption!(data_reader.read_u32());
            let b_flags = tryOption!(data_reader.read_i16());

            blocks.push(ArchiveBlockInfo {
                            uncompressed_size: bu_size,
                            compressed_size: bc_size,
                            flags: b_flags,
                        });
        }

        let num_nodes = tryOption!(data_reader.read_u32());
        let mut nodes: Vec<(u64, u64, u32, String)> = vec![];
        for _ in 0..num_nodes {
            let n_offset = tryOption!(data_reader.read_u64());
            let n_size = tryOption!(data_reader.read_u64());
            let n_status = tryOption!(data_reader.read_u32());
            let n_name = tryOption!(data_reader.read_string());
            nodes.push((n_offset, n_size, n_status, n_name));
        }

        let mut storageReader = ArchiveBlockStorageReader::new(buffer.take_buffer(), blocks);
        for (n_offset, n_size, n_status, n_name) in nodes {
            storageReader.seek(SeekFrom::Start(n_offset));
        }

        None
    }

    fn load_raw(&mut self) -> Option<Error> {
        // TODO: loading UnityWeb | UnityRaw format
        Some(Error::new(ErrorKind::InvalidData, "UnityWeb format is not implemented"))
    }

    fn load_unityarchive(&mut self) -> Option<Error> {
        // TODO: loading UnityArchive format
        Some(Error::new(ErrorKind::InvalidData,
                        "UnityArchive format is not implemented"))
    }
}

/// Contains compression information about a block
struct ArchiveBlockInfo {
    uncompressed_size: u32,
    compressed_size: u32,
    flags: i16,
}

impl ArchiveBlockInfo {
    fn compression_type(&self) -> CompressionType {
        let flag = (self.flags as u8 & 0x3f) as u8;
        CompressionType::from(&flag)
    }

    fn is_compressed(&self) -> bool {
        self.compression_type() != CompressionType::None
    }

    fn decompress(&self, data: Vec<u8>) -> io::Result<Vec<u8>> {
        if !self.is_compressed() {
            return Ok(data);
        }

        let compression_type = self.compression_type();
        match compression_type {
            CompressionType::LZMA => {
                // TODO: LZMA decompression
                Ok(data)
            }
            CompressionType::LZ4 |
            CompressionType::LZ4HC => {
                let decompressed_data = vec![0; self.uncompressed_size as usize];
                decompress_data(&decompressed_data, &compression_type)
            }
            _ => {
                Err(Error::new(ErrorKind::InvalidData,
                               format!("Unimplemented compression method: {:?}", compression_type)))
            }
        }
    }
}

/// ArchiveBlockStorageReader reads data that is composed of compressed blocks
struct ArchiveBlockStorageReader<R: Read + Seek> {
    /// Read object for the underlying compressed blocks
    buffer: BufReader<R>,
    blocks: Vec<ArchiveBlockInfo>,
    /// total uncompressed size
    virtual_size: u64,
    /// cursor in the virtual uncompressed buffer
    virtual_cursor: u64,
    /// offset of the virtual block in buffer
    base_offset: u64,
    /// points to the currently decompressed block
    current_block_idx: isize,
    /// offset to the current block in the virtual buffer
    current_block_offset: u64,
    /// current uncompressed block
    current_stream: Vec<u8>,
}

impl<R> ArchiveBlockStorageReader<R>
    where R: Read + Seek
{
    fn new(mut buffer: BufReader<R>,
           blocks: Vec<ArchiveBlockInfo>)
           -> ArchiveBlockStorageReader<R> {
        let virtual_size = blocks
            .iter()
            .fold(0, |total, next| total + next.uncompressed_size as u64);

        let base_offset = buffer.tell();

        ArchiveBlockStorageReader {
            virtual_cursor: 0,
            buffer: buffer,
            blocks: blocks,
            virtual_size: virtual_size,
            base_offset: base_offset,
            current_block_idx: -1 as isize,
            current_block_offset: 0,
            current_stream: Vec::new(),
        }
    }

    fn seek_to_block(&mut self, pos: &u64) -> io::Result<()> {
        // check if we are already in the corresponding block
        if (self.current_block_idx < 0) ||
           ((*pos < self.current_block_offset) ||
            (*pos >
             (self.current_block_offset +
              self.blocks[self.current_block_idx as usize].uncompressed_size as u64))) {
            let mut base_offset: u64 = 0;
            let mut offset = 0;
            let mut found = false;
            for b in 0..self.blocks.len() {
                let block = &self.blocks[b];
                if offset + block.uncompressed_size as u64 > *pos {
                    self.current_block_idx = b as isize;
                    found = true;
                    break;
                }
                base_offset += block.compressed_size as u64;
                offset += block.uncompressed_size as u64;
            }

            if !found {
                self.current_block_idx = -1;
                self.current_stream = Vec::new();
                return Ok(());
            }

            self.current_block_offset = offset;
            try!(self.buffer
                     .seek(SeekFrom::Start(self.base_offset + base_offset)));
            let current_block = &self.blocks[self.current_block_idx as usize];
            let mut compressed_data = vec![0; current_block.compressed_size as usize];
            try!(self.buffer.read_exact(compressed_data.as_mut_slice()));
            self.current_stream = try!(current_block.decompress(compressed_data));
        }
        Ok(())
    }
}

impl<R> Read for ArchiveBlockStorageReader<R>
    where R: Read + Seek
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

        let mut size = buf.len();
        let mut bytes: Vec<u8> = Vec::new();

        while size != 0 && self.virtual_cursor < self.virtual_size {
            let cursor = self.virtual_cursor;
            try!(self.seek_to_block(&cursor));

            let current_stream_cursor = self.virtual_cursor - self.current_block_offset;
            let current_stream_len = self.current_stream.len();
            if (current_stream_len as u64) < current_stream_cursor {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Error while reading block storeage"));
            }
            let remaining = (current_stream_len as u64) - current_stream_cursor;
            let read_size = cmp::min(size, remaining as usize);
            if read_size == 0 {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Error while reading block storeage"));
            }
            let part = &self.current_stream[(current_stream_cursor as usize)..read_size];
            size -= read_size;
            self.virtual_cursor += read_size as u64;
            bytes.extend(part);
        }
        buf.clone_from_slice(&bytes);
        Ok(bytes.len())
    }
}

impl<R> Teller for ArchiveBlockStorageReader<R>
    where R: Read + Seek
{
    fn tell(&mut self) -> u64 {
        self.virtual_cursor
    }
}

impl<R> Seek for ArchiveBlockStorageReader<R>
    where R: Read + Seek
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos: u64;
        match pos {
            SeekFrom::Start(p) => {
                new_pos = p;
            }
            SeekFrom::End(p) => {
                if p < 0 {
                    new_pos = self.virtual_size - (p.abs() as u64);
                } else {
                    new_pos = self.virtual_size + (p as u64);
                }
            }
            SeekFrom::Current(p) => {
                if p < 0 {
                    new_pos = self.virtual_cursor - (p.abs() as u64);
                } else {
                    new_pos = self.virtual_cursor + (p as u64);
                }
            }
        };

        try!(self.seek_to_block(&new_pos));
        self.virtual_cursor = new_pos;
        Ok(self.virtual_cursor)
    }
}

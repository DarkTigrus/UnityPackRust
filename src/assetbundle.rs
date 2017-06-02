/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved
 */
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::BufReader;
use binaryreader::*;
use lz4_compress;

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
}

impl CompressionType {
	fn from( x: &u8 ) -> Result<CompressionType, Error> {
		match x {
			x if *x == CompressionType::None as u8 => Ok(CompressionType::None),
    		x if *x == CompressionType::LZMA as u8 => Ok(CompressionType::LZMA),
    		x if *x == CompressionType::LZ4 as u8 => Ok(CompressionType::LZ4),
			x if *x == CompressionType::LZ4HC as u8 => Ok(CompressionType::LZ4HC),
			x if *x == CompressionType::LZHAM as u8 => Ok(CompressionType::LZHAM),
    		_ => Err(Error::new(ErrorKind::InvalidData, format!("Unknown compression type found: {}",x)  )),
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
}

impl Default for AssetBundle {
    fn default() -> AssetBundle {
		AssetBundle {
			signature: Signature::Unknown,
			format_version: 0,
			target_version: String::new(),
			generator_version: String::new(),
			descriptor: FSDescriptor::Unknown, }
	}
}

struct ArchiveBlockInfo {
	uncompressed_size: u32,
	compressed_size: u32,
	flags: i16,
}

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

fn decompress_data(data: &Vec<u8>, compression_type: &CompressionType) -> Result<Vec<u8>,Error> {
	match *compression_type {
		CompressionType::LZ4|CompressionType::LZ4HC => {
			println!("{:?}",data);
			match lz4_compress::decompress(data.as_slice()) {
				Err(err) => {return Err(Error::new(ErrorKind::InvalidData, format!("LZ4 decompression failed: {:?}",err) )); },
				Ok(buf) => {Ok(buf)},
			}
		},
		CompressionType::LZMA|CompressionType::LZHAM => Err(Error::new(ErrorKind::InvalidData, format!("{:?} is not yet implemented",*compression_type) )),
		_ => Ok(data.clone()),
	}
}

impl AssetBundle {
	
	pub extern fn load_from_file(file_path: &str) -> Result<AssetBundle, Error> {
		
		// open file
		let file = try!(File::open(file_path));
		let mut buf_reader = BufReader::new(file);
		let mut bin_reader = BinaryReader::new(&mut buf_reader, Endianness::Big);

		let mut result = AssetBundle { signature: Signature::Unknown,
									   format_version: 0,
									   target_version: String::new(),
									   generator_version: String::new(),
									   descriptor: FSDescriptor::Unknown,
									 };

		// read header
		let signature_str = try!(bin_reader.read_string());
		if let Ok(x) = signature_str.parse() {
			result.signature = x;
		} else {
			result.signature = Signature::Unknown;
		}

		match result.signature {
			Signature::UnityArchive => { result.load_unityarchive(); }
			Signature::UnityFS => { isOptionError!(result.load_unityfs(&mut bin_reader)); }
			Signature::UnityWeb|Signature::UnityRaw => { result.load_raw(); }
			_ => { return Err(Error::new(ErrorKind::InvalidData, format!("Unknown format found: {}", signature_str) )); }
		}
		
		Ok(result)
	}

	fn load_unityfs(&mut self, buffer: &mut BinaryReader) -> Option<Error> {
		self.format_version = tryOption!(buffer.read_u32());
		self.target_version = tryOption!(buffer.read_string());
		self.generator_version = tryOption!(buffer.read_string());

		let file_size = tryOption!(buffer.read_i64());
		let ciblock_size = tryOption!(buffer.read_u32());
		let uiblock_size = tryOption!(buffer.read_u32());

		self.descriptor = FSDescriptor::UnityFs(UnityFsDescriptor {
			fs_file_size: file_size,
			ci_block_size: ciblock_size,
			ui_block_size: uiblock_size });

        let flags = (tryOption!(buffer.read_u32()) as u8) & 0x3F;
		let compression_type = tryOption!(CompressionType::from(&flags));
		let raw_data = tryOption!(buffer.read_bytes(&(ciblock_size as u64)));

		let decompressed_data = tryOption!(decompress_data(&raw_data, &compression_type));
		let mut decompressed_data_array = decompressed_data.as_slice();
		let mut data_reader = BinaryReader::new(&mut decompressed_data_array, Endianness::Big);
		
		tryVoid!(data_reader.read_bytes(&16)); // guid
		
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
		

		None
	}

	fn load_raw(&mut self) -> Option<Error> {
		// TODO: loading UnityWeb | UnityRaw format
		Some(Error::new(ErrorKind::InvalidData, "UnityWeb format is not implemented" ))
	}

	fn load_unityarchive(&mut self) -> Option<Error> {
		// TODO: loading UnityArchive format
		Some(Error::new(ErrorKind::InvalidData, "UnityArchive format is not implemented" ))
	}
}	

	

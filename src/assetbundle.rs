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
use std::io::Cursor;
use std::io::Read;
use binaryreader::*;
use lz4::Decoder;

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

pub struct AssetBundle {
	signature: Signature,
	format_version: u32,
	target_version: String, // also called unity_version
	generator_version: String,
	descriptor: FSDescriptor,
}

macro_rules! tryVoid {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => return Some(err),
    });
}

fn decompress_data(data: &Vec<u8>, compression_type: &CompressionType, uncompressed_size: &u32) -> Result<Vec<u8>,Error> {
	match *compression_type {
		CompressionType::LZ4|CompressionType::LZ4HC => {
			let mut in_buf = Cursor::new(data);
			let mut decoder = try!(Decoder::new(in_buf));
			let mut out_buf = vec![];
			let data_read = try!(decoder.read_to_end(&mut out_buf));
			Ok(out_buf)
		}
		_ => Ok(data.clone()),
	}
}

impl AssetBundle {
	
	#[no_mangle]
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
			Signature::UnityFS => { result.load_unityfs(&mut bin_reader); }
			Signature::UnityWeb|Signature::UnityRaw => { result.load_raw(); }
			_ => { return Err(Error::new(ErrorKind::InvalidData, format!("Unknown format found: {}", signature_str) )); }
		}
		
		Ok(result)
	}

	fn load_unityfs(&mut self, buffer: &mut BinaryReader) -> Option<Error> {
		self.format_version = tryVoid!(buffer.read_u32());
		self.target_version = tryVoid!(buffer.read_string());
		self.generator_version = tryVoid!(buffer.read_string());

		let file_size = tryVoid!(buffer.read_i64());
		let ciblock_size = tryVoid!(buffer.read_u32());
		let uiblock_size = tryVoid!(buffer.read_u32());

		self.descriptor = FSDescriptor::UnityFs(UnityFsDescriptor {
			fs_file_size: file_size,
			ci_block_size: ciblock_size,
			ui_block_size: uiblock_size });

		/*if let FSDescriptor::UnityFs(ref d) = self.descriptor {
			println!("{} {} {}",d.fs_file_size, d.ci_block_size, d.ui_block_size);
		}*/
        let flags = (tryVoid!(buffer.read_u32()) as u8) & 0x3F;
		let compression_type = tryVoid!(CompressionType::from(&flags));

		let raw_data = tryVoid!(buffer.read_bytes(&(ciblock_size as u64)));
		let decompressed_data = tryVoid!(decompress_data(&raw_data, &compression_type, &uiblock_size));
		
		
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

	

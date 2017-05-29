/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved
 */
use std::fs::File;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::BufReader;
use binaryreader::*;

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

pub struct AssetBundle {
	signature: Signature,
}

impl AssetBundle {
	
	#[no_mangle]
	pub extern fn load_from_file(file_path: &str) -> io::Result<AssetBundle> {
		
		// open file
		let file = File::open(file_path)?;
		let buf_reader = BufReader::new(file);
		let mut bin_reader = BinaryReader::new(buf_reader, Endianness::Big);

		let mut result = AssetBundle {signature: Signature::Unknown};

		// read header
		let signature_str = bin_reader.read_string();
		if let Ok(x) = signature_str.parse() {
			result.signature = x;
		} else {
			result.signature = Signature::Unknown;
		}

		match result.signature {
			Signature::UnityArchive => { result.load_unityarchive(); }
			Signature::UnityFS => { result.load_unityfs(); }
			Signature::UnityWeb|Signature::UnityRaw => { result.load_raw(); }
			_ => { return Err(Error::new(ErrorKind::InvalidData, format!("Unknown format found: {}", signature_str) )); }
		}
		
		Ok(result)
	}

	fn load_unityfs(&mut self) {
		// TODO: loading UnityFS format
	}

	fn load_raw(&mut self) {
		// TODO: loading UnityWeb | UnityRaw format
	}

	fn load_unityarchive(&mut self) {
		// TODO: loading UnityArchive format
	}
}	

	

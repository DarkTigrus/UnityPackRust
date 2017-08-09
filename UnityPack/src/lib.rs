/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
extern crate libc;
extern crate byteorder;
extern crate lz4_compress;
extern crate lzma;
extern crate xz2;
extern crate uuid;
extern crate serde_json;
extern crate odds;

#[macro_use]
extern crate lazy_static;

mod error;
mod assetbundle;
mod asset;
mod object;
mod binaryreader;
mod typetree;
mod enums;
mod resources;
pub mod unitypack_c;
mod extras;

#[cfg(test)]
mod tests {

    use assetbundle::*;

    #[test]
    fn test_load_assetbundle() {
        let input_file = "test_data/main_dxt1_bc1.unity3d";

        let asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                assert!(false);
                return;
            }
        };

        //println!("sig: {:?}", asset_bundle.signature);

    }

}

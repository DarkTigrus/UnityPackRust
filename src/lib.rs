/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
extern crate libc;
extern crate byteorder;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;
extern crate lz4_compress;

mod assetbundle;
mod asset;
mod binaryreader;
pub mod unitypack_c;

#[cfg(test)]
mod tests {

    use assetbundle::*;

    #[test]
    fn test_load_assetbundle() {
        let input_file = "/Applications/Hearthstone/Data/OSX/cards0.unity3d";

        let asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                assert!(false);
                return;
            }
        };

    }

}

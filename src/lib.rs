/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved
 */
extern crate byteorder;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;

pub mod assetbundle;
mod binaryreader;

#[cfg(test)]
mod tests {

    use assetbundle::*;
	
    #[test]
    fn test_load_assetbundle() {
    	let input_file = "/Applications/Hearthstone/Data/OSX/cards0.unity3d";
	
		let assetbundle = AssetBundle::load_from_file(input_file);
	    println!("{}", input_file);
    }
}



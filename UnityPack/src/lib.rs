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
extern crate lzma_sys;
extern crate uuid;
extern crate serde_json;
extern crate bcndecode;
extern crate decrunch;

#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod assetbundle;
pub mod asset;
pub mod object;
mod binaryreader;
mod typetree;
mod enums;
mod resources;
pub mod unitypack_c;
mod extras;
pub mod engine;

#[cfg(test)]
mod tests {

    use assetbundle::*;
    use object::*;
    use engine::texture::IntoTexture2D;
    use engine::text::IntoTextAsset;

    #[test]
    fn test_load_texture2d() {
        let input_file = "test_data/main_dxt1_bc1.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                assert!(false);
                return;
            }
        };

        assert_eq!(asset_bundle.assets.len(), 1);
        asset_bundle.resolve_asset(0).unwrap();
        let asset = &asset_bundle.assets[0];

        assert_eq!(asset.name, "CAB-ba01e3c16ba268ec36e9543a39dc83ad");

        let objects = &asset.objects;
        assert_eq!(objects.len(), 4);

        for obj in objects.values() {
            if obj.type_name == "Texture2D" {
                let engine_object = obj.read_signature(asset, &mut asset_bundle.signature)
                    .unwrap();
                let texture = match engine_object {
                    ObjectValue::EngineObject(engine_object) => {
                        engine_object.to_texture2d().unwrap()
                    }
                    _ => {
                        panic!("Invalid engine object");
                    }
                };

                println!(
                    "{}: {} ({}x{}) - {} bytes, format: {:?}",
                    obj.type_name,
                    texture.name,
                    texture.width,
                    texture.height,
                    texture.data.len(),
                    texture.texture_format
                );

                let _ = texture.to_image().unwrap();

            }
        }
    }

    #[test]
    fn test_load_gameobjects() {
        let input_file = "/Applications/Hearthstone/Data/OSX/gameobjects0.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                assert!(false);
                return;
            }
        };

        assert!(asset_bundle.assets.len() > 0);

        for i in 0..asset_bundle.assets.len() {
            asset_bundle.resolve_asset(i).unwrap();
        }

    }

    #[test]
    fn test_load_textasset() {
        let input_file = "/Applications/Hearthstone/Data/OSX/cardxml0.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                assert!(false);
                return;
            }
        };

        assert!(asset_bundle.assets.len() > 0);
        asset_bundle.resolve_asset(0).unwrap();
        let asset = &asset_bundle.assets[0];
        let objects = &asset.objects;

        for obj in objects.values() {
            if obj.type_name == "TextAsset" {
                let engine_object = obj.read_signature(asset, &mut asset_bundle.signature)
                    .unwrap();
                let _ = match engine_object {
                    ObjectValue::EngineObject(engine_object) => {
                        engine_object.to_textasset().unwrap()
                    }
                    _ => {
                        panic!("Invalid engine object");
                    }
                };
                // println!("{}",text.script); too long
            }
        }

    }

}

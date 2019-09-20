/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

extern crate bcndecode;
extern crate byteorder;
extern crate decrunch;
extern crate libc;
extern crate lz4_compress;
extern crate lzma;
extern crate lzma_sys;
extern crate serde_json;
extern crate uuid;

#[macro_use]
extern crate lazy_static;

pub mod asset;
pub mod assetbundle;
mod binaryreader;
pub mod engine;
mod enums;
pub mod error;
mod extras;
pub mod object;
mod resources;
mod typetree;
pub mod unitypack_c;

#[cfg(test)]
mod tests {

    use assetbundle::*;
    use engine::font::IntoFont;
    use engine::font::IntoFontDef;
    use engine::mesh::IntoMesh;
    use engine::text::IntoTextAsset;
    use engine::texture::IntoTexture2D;
    use object::*;

    #[test]
    fn test_load_texture2d() {
        let input_file = "test_data/main_dxt1_bc1.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                panic!();
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
                let engine_object = obj
                    .read_signature(asset, &mut asset_bundle.signature)
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
                panic!();
            }
        };

        assert!(asset_bundle.assets.is_empty(), false);

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
                panic!();
            }
        };

        assert!(asset_bundle.assets.is_empty(), false);
        asset_bundle.resolve_asset(0).unwrap();
        let asset = &asset_bundle.assets[0];
        let objects = &asset.objects;

        for obj in objects.values() {
            if obj.type_name == "TextAsset" {
                let engine_object = obj
                    .read_signature(asset, &mut asset_bundle.signature)
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

    #[test]
    fn test_load_fontdef() {
        let input_file = "/Applications/Hearthstone/Data/OSX/fonts0.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                panic!();
            }
        };

        assert!(asset_bundle.assets.is_empty(), false);
        asset_bundle.resolve_asset(0).unwrap();

        let asset = &asset_bundle.assets[0];
        let objects = &asset.objects;

        for obj in objects.values() {
            if obj.type_name == "FontDef" {
                let engine_object = obj
                    .read_signature(asset, &mut asset_bundle.signature)
                    .unwrap();

                let _ = match engine_object {
                    ObjectValue::EngineObject(engine_object) => {
                        engine_object.to_fontdef(&asset).unwrap()
                    }
                    _ => {
                        panic!("Invalid engine object: {:?}", engine_object);
                    }
                };
            }
        }
    }

    #[test]
    fn test_load_font() {
        let input_file = "/Applications/Hearthstone/Data/OSX/shared1.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                panic!();
            }
        };

        assert!(asset_bundle.assets.is_empty(), false);
        asset_bundle.resolve_asset(0).unwrap();

        let asset = &asset_bundle.assets[0];
        let objects = &asset.objects;

        for obj in objects.values() {
            if obj.type_name == "Font" {
                let engine_object = obj
                    .read_signature(asset, &mut asset_bundle.signature)
                    .unwrap();

                let _ = match engine_object {
                    ObjectValue::EngineObject(engine_object) => engine_object.to_font().unwrap(),
                    _ => {
                        panic!("Invalid engine object: {:?}", engine_object);
                    }
                };
            }
        }
    }

    #[test]
    fn test_load_mesh() {
        let input_file = "/Applications/Hearthstone/Data/OSX/actors0.unity3d";

        let mut asset_bundle = match AssetBundle::load_from_file(input_file) {
            Ok(f) => f,
            Err(err) => {
                println!("Failed to load assetbundle from {}", input_file);
                println!("Error: {:?}", err);
                panic!();
            }
        };

        assert!(asset_bundle.assets.is_empty(), false);
        asset_bundle.resolve_asset(0).unwrap();

        let asset = &asset_bundle.assets[0];
        let objects = &asset.objects;

        for obj in objects.values() {
            if obj.type_name == "Mesh" {
                let engine_object = obj
                    .read_signature(asset, &mut asset_bundle.signature)
                    .unwrap();

                let _ = match engine_object {
                    ObjectValue::EngineObject(engine_object) => engine_object.to_mesh().unwrap(),
                    _ => {
                        panic!("Invalid engine object: {:?}", engine_object);
                    }
                };
            }
        }
    }

}

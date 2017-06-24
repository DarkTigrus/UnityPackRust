/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::mem::transmute;
use assetbundle::*;
use asset::*;
use std::ptr;
use libc;
use libc::uint32_t;

// C API
#[no_mangle]
pub extern "C" fn unitypack_load_assetbundle_from_file(file_path: *const c_char)
                                                       -> *const libc::c_void {
    let file_path_str = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };

    unsafe {
        let abundle: AssetBundle = match AssetBundle::load_from_file(file_path_str) {
            Ok(assetbundle) => assetbundle,
            _ => {
                println!("error loading assetbundle");
                return ptr::null();
            }
        };
        transmute(Box::new(abundle))
    }
}

#[no_mangle]
pub extern "C" fn unitypack_destroy_assetbundle(ptr: *mut AssetBundle) {
    let _bundle: Box<AssetBundle> = unsafe { transmute(ptr) };
    // Drop
}

#[no_mangle]
pub extern "C" fn unitypack_get_num_assets(ptr: *mut AssetBundle) -> uint32_t {
    let mut _bundle = unsafe { &mut *ptr };
    let assets = _bundle.assets();
    assets.len() as uint32_t
}

#[no_mangle]
pub extern "C" fn unitypack_get_asset(ptr: *mut AssetBundle, i: uint32_t) -> *const libc::c_void {
    let mut _bundle = unsafe { &mut *ptr };
    let assets = _bundle.assets();
    if i as usize >= assets.len() {
        return ptr::null();
    }

    unsafe {
        let asset_ptr: *const Asset = &assets[i as usize];
        //transmute(Box::new(asset_ptr))
        //asset_ptr
        transmute(asset_ptr)
    }
}

#[no_mangle]
pub extern "C" fn unitypack_get_asset_name(ptr: *mut Asset) -> *mut c_char {
    let mut _asset = unsafe { &mut *ptr };

    let c_str_name = CString::new(_asset.name.clone()).unwrap();
    c_str_name.into_raw()
}

#[no_mangle]
pub extern "C" fn unitypack_free_rust_string(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}

#[no_mangle]
pub extern "C" fn unitypack_get_num_objects(ptr: *mut Asset) -> uint32_t {
    let mut _asset = unsafe { &mut *ptr };

    let objects = _asset.get_objects();
    objects.len() as uint32_t
}

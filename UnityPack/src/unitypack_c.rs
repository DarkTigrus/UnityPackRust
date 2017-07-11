/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::mem::{transmute, forget};
use assetbundle::*;
use asset::*;
use object::ObjectInfo;
use std::ptr;
use libc;
use libc::uint32_t;

#[repr(C)]
pub struct ObjectArray {
    array: *const libc::c_void,
    length: libc::size_t,
}

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
pub extern "C" fn unitypack_get_num_objects(asset_ptr: *mut Asset, bundle_ptr: *mut AssetBundle) -> uint32_t {
    let mut _asset = unsafe { &mut *asset_ptr };
    let mut _bundle = unsafe { &mut *bundle_ptr };

    let objects = match _asset.get_objects(_bundle) {
        Ok(obj) => obj,
        Err(err) => {
            println!("{}",err);
            return 0;
        },
    };
    objects.len() as uint32_t
}

#[no_mangle]
pub extern "C" fn unitypack_get_objects_with_type(asset_ptr: *mut Asset, bundle_ptr: *mut AssetBundle, obj_type: *const c_char)  -> ObjectArray {
    let mut _asset = unsafe { &mut *asset_ptr };
    let mut _bundle = unsafe { &mut *bundle_ptr };

    let obj_type_str = unsafe { CStr::from_ptr(obj_type).to_str().unwrap() };

    let objects = match _asset.get_objects(_bundle) {
        Ok(obj) => obj,
        Err(err) => {
            println!("{}",err);
            return ObjectArray {
                array: ptr::null(),
                length: 0,
            };
        },
    };

    let mut v: Vec<*const libc::c_void> = Vec::new();

    for obj in objects.values() {
        if obj.get_type() == obj_type_str {
            let obj_ptr: *const ObjectInfo = obj;
            unsafe {
                v.push(transmute(obj_ptr));
            }
        }
    }

    let boxed_slice: Box<[*const libc::c_void]> = v.into_boxed_slice();

    let result = ObjectArray {
        array: boxed_slice.as_ptr() as *const libc::c_void,
        length: boxed_slice.len() as _,
    };

    forget(boxed_slice);
        
    result
}

/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use std::ffi::CStr;
use std::os::raw::c_char;
use std::mem::transmute;
use assetbundle::*;
use std::ptr;
use libc;

// C API
#[no_mangle]
pub extern "C" fn load_assetbundle_from_file(file_path: *const c_char) -> *const libc::c_void {
    let file_path_str = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };

    unsafe {
        let abundle: AssetBundle = match AssetBundle::load_from_file(file_path_str) {
            Ok(assetbundle) => assetbundle,
            _ => return ptr::null(),
        };
        transmute(Box::new(abundle))
    }
}

#[no_mangle]
pub extern "C" fn destroy_assetbundle(ptr: *mut AssetBundle) {
    let _bundle: Box<AssetBundle> = unsafe { transmute(ptr) };
    // Drop
}

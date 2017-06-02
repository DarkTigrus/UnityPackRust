use std::ffi::CStr;
use std::os::raw::c_char;
use std::mem::transmute;
use assetbundle::*;

// C API
#[no_mangle]
pub extern fn load_assetbundle_from_file(file_path: *const c_char) -> *mut AssetBundle {
	let file_path_str = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };
	
    let _bundle = unsafe {
		let abundle: AssetBundle = match AssetBundle::load_from_file(file_path_str) {
			Ok(assetbundle) => assetbundle,
			_ => Default::default(),
		};
		transmute(Box::new(abundle))
	};
    _bundle
}

#[no_mangle]
pub extern fn destroy_assetbundle(ptr: *mut AssetBundle) {
    let _bundle: Box<AssetBundle> = unsafe{ transmute(ptr) };
    // Drop
}
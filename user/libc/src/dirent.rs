use alloc::{alloc::alloc, boxed::Box};
use core::{
	alloc::Layout,
	ffi::{c_char, CStr},
};

use crate::{api, api::__dirstream, syscall};

#[no_mangle]
pub extern "C" fn opendir(path: *const c_char) -> *mut api::DIR {
	let c_str = unsafe { CStr::from_ptr(path) }.to_bytes();
	let fd = syscall::open(c_str.as_ptr(), c_str.len());
	Box::into_raw(Box::new(__dirstream { fd }))
}

#[no_mangle]
pub extern "C" fn readdir(dir: *const api::DIR) -> *mut api::dirent {
	let fd = unsafe { &*dir }.fd;
	let entry =
		unsafe { alloc(Layout::new::<api::dirent>()) } as *mut api::dirent;
	syscall::readdir(fd, entry);
	entry
}

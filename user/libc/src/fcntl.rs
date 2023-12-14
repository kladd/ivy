use core::ffi::{c_char, c_int, CStr};

use crate::syscall;

#[no_mangle]
pub extern "C" fn open(path: *const c_char, _oflag: c_int) -> c_int {
	let fname = unsafe { CStr::from_ptr(path).to_bytes() };
	syscall::open(fname.as_ptr(), fname.len()) as c_int
}

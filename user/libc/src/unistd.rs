use core::ffi::{c_char, c_int, c_void, CStr};

use crate::syscall;

#[no_mangle]
pub fn chdir(path: *const c_char) -> c_int {
	let path = unsafe { CStr::from_ptr(path) }
		.to_str()
		.expect("utf8 error");
	syscall::chdir(path.as_ptr(), path.len()) as c_int
}

#[no_mangle]
pub fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
	syscall::syscall2(11, buf as u64, size as u64);
	buf
}

#[no_mangle]
pub extern "C" fn read(fildes: c_int, buf: *mut c_void, nbyte: usize) -> isize {
	syscall::read(fildes as u64, buf as *mut u8, nbyte) as isize
}

#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *const c_void, len: usize) -> isize {
	syscall::write(fd as u64, buf as *const u8, len);
	len as isize
}

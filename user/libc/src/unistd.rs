use core::ffi::{c_int, c_void};

use crate::syscall;

#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *const c_void, len: usize) -> isize {
	syscall::write(fd as u64, buf as *const u8, len);
	len as isize
}

#[no_mangle]
pub extern "C" fn read(fildes: c_int, buf: *mut c_void, nbyte: usize) -> isize {
	syscall::read(fildes as u64, buf as *mut u8, nbyte) as isize
}

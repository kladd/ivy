use core::ffi::c_int;

use crate::{api, syscall};

#[no_mangle]
pub extern "C" fn fstat(fildes: c_int, buf: *mut api::stat) -> c_int {
	syscall::syscall2(10, fildes as u64, buf as u64) as c_int
}

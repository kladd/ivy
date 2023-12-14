use crate::syscall;

pub extern "C" fn write(
	fd: ::core::ffi::c_int,
	buf: *const ::core::ffi::c_void,
	len: usize,
) -> isize {
	syscall::write(fd as u64, buf as *const u8, len);
	len as isize
}

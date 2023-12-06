#![no_std]
#![no_main]

use sys::syscall::video_test;

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	video_test();
	0
}

#![no_std]
#![no_main]

use sys::syscall::{brk, putc, video_test};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	let ret = brk(0);

	putc((ret as u8 + '0' as u8) as char);
	video_test();

	ret as isize
}

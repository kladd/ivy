#![no_std]
#![feature(start)]

use sys::syscall::{exit, video_test};

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	video_test();
	exit();
}

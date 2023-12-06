#![no_std]
#![no_main]

use core::arch::global_asm;

use sys::syscall::{exit, video_test};

global_asm! {
	".global _start",
	"_start:",
	"call main"
}

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	video_test();
	exit();
}

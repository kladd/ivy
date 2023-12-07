#![no_std]
#![no_main]

extern crate alloc;

use sys::syscall::{
	brk, debug_long, print_line, read_line, video_clear, video_test, write,
};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	let current_break = brk(0);
	debug_long(current_break as u64);

	video_clear();
	loop {
		let line = read_line();

		match line.as_str() {
			"test" => video_test(),
			"clear" => video_clear(),
			"exit" => break,
			_ => print_line(&line),
		}
	}

	0
}

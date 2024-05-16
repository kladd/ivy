#![no_std]
#![no_main]

use libc::{api::STDOUT_FILENO, unistd::write};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	let message = "hello, world\n";
	write(STDOUT_FILENO, message.as_ptr() as *const _, message.len());
	0
}

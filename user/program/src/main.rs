#![no_std]
#![no_main]

extern crate alloc;

use alloc::alloc::alloc;
use core::{alloc::Layout, slice, str};

use sys::syscall::{brk, debug_long, read, write};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	let current_break = brk(0);
	debug_long(current_break as u64);

	let buf = unsafe { alloc(Layout::array::<u8>(80).unwrap()) };
	loop {
		let len = read(0, buf, 80);
		let line = unsafe {
			str::from_utf8_unchecked(slice::from_raw_parts(buf, len))
		};

		match line {
			"exit" => break,
			"" => continue,
			_ => {
				write(0, line.as_ptr(), line.len());
				write(0, "\n".as_ptr(), 1);
			}
		}
	}

	0
}

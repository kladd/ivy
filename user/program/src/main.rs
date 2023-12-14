#![no_std]
#![no_main]

extern crate alloc;

use alloc::alloc::alloc;
use core::{alloc::Layout, slice, str};

use sys::syscall::{open, read, stat, write};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	let buf = unsafe { alloc(Layout::array::<u8>(80).unwrap()) };

	let tty = "/dev/tty0";
	let fd = open(tty.as_ptr(), tty.len());

	if fd < 0 {
		return fd;
	}
	stat(fd);

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

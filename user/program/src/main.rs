#![no_std]
#![no_main]

extern crate alloc;

use alloc::{ffi::CString, format};
use core::ffi::{c_void, CStr};

use libc::{
	dirent::{opendir, readdir},
	fcntl::open,
	unistd::write,
};

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	// let tty = "/dev/tty0";
	// let fd = open(tty.as_ptr(), tty.len());
	//
	//
	//
	// if fd < 0 {
	// 	return fd;
	// }
	// stat(fd);
	//
	// loop {
	// 	let len = read(0, buf, 80);
	// 	let line = unsafe {
	// 		str::from_utf8_unchecked(slice::from_raw_parts(buf, len))
	// 	};
	//
	// 	match line {
	// 		"exit" => break,
	// 		"" => continue,
	// 		_ => {
	// 			write(0, line.as_ptr(), line.len());
	// 			write(0, "\n".as_ptr(), 1);
	// 		}
	// 	}
	// }
	//
	// 0

	let console = CString::new("/dev/tty0").unwrap();
	let fd_cons = open(console.as_ptr(), 0);

	let root = CString::new("/").unwrap();
	let fd_root = opendir(root.as_ptr());

	loop {
		let entry = unsafe { &*readdir(fd_root) };
		if entry.d_ino == 0 {
			break;
		}
		let name = unsafe { CStr::from_ptr(entry.d_name.as_ptr()) }
			.to_str()
			.unwrap();
		let name = format!("{name}\n");

		write(fd_cons, name.as_ptr() as *const c_void, name.len());
	}

	0
}

#![no_std]
#![no_main]

extern crate alloc;

use alloc::{ffi::CString, format, string::ToString};
use core::{
	ffi::{c_void, CStr},
	slice, str,
};

use libc::{
	api::{STDIN_FILENO, STDOUT_FILENO},
	dirent::{opendir, readdir},
	syscall,
	unistd::{read, write},
};

fn shell() {
	let mut line_buf = [0u8; 128];
	loop {
		write(STDOUT_FILENO, "@ ".as_ptr() as *const c_void, 2);
		let len = read(
			STDIN_FILENO,
			line_buf.as_mut_ptr() as *mut c_void,
			line_buf.len(),
		);
		let cmdline = unsafe {
			str::from_utf8_unchecked(slice::from_raw_parts(
				line_buf.as_ptr(),
				len as usize,
			))
		};
		let mut tokens = cmdline.split_ascii_whitespace();
		match tokens.next() {
			Some("ls") => ls(),
			Some("exit") => break,
			Some("uptime") => uptime(),
			_ => continue,
		}
	}
}

fn uptime() {
	let time = format!("{}\n", syscall::uptime());
	write(STDOUT_FILENO, time.as_ptr() as *const c_void, time.len());
}

fn ls() {
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

		write(STDOUT_FILENO, name.as_ptr() as *const c_void, name.len());
	}
}

#[no_mangle]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
	shell();
	0
}

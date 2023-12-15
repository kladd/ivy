#![no_std]
#![no_main]

extern crate alloc;

use alloc::{ffi::CString, format, vec};
use core::{
	ffi::{c_char, c_void, CStr},
	slice, str,
};

use libc::{
	api::{getcwd, STDIN_FILENO, STDOUT_FILENO},
	dirent::{opendir, readdir},
	fcntl::open,
	syscall,
	unistd::{chdir, read, write},
};

fn shell() {
	let mut cwd_buf = [0 as c_char; 128];
	unsafe { getcwd(cwd_buf.as_mut_ptr(), cwd_buf.len()) };
	let mut prompt = format!(
		"[{}]$ ",
		unsafe { CStr::from_ptr(cwd_buf.as_ptr()) }
			.to_str()
			.unwrap()
	);

	let mut line_buf = [0u8; 128];
	loop {
		write(
			STDOUT_FILENO,
			prompt.as_ptr() as *const c_void,
			prompt.len(),
		);
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
			Some("ls") => ls(tokens.next()),
			Some("exit") => break,
			Some("uptime") => uptime(),
			Some("cd") => {
				cd(tokens.next());
				cwd_buf.iter_mut().for_each(|e| *e = 0);
				unsafe { getcwd(cwd_buf.as_mut_ptr(), cwd_buf.len()) };
				prompt = format!(
					"[{}]$ ",
					unsafe { CStr::from_ptr(cwd_buf.as_ptr()) }
						.to_str()
						.unwrap()
				);
			}
			Some("cat") => cat(tokens.next()),
			_ => continue,
		}
	}
}

fn cat(path: Option<&str>) {
	let Some(path) = path else { return };
	let file = open(CString::new(path).unwrap().as_ptr(), 0);
	if file < 0 {
		return;
	}
	let mut stat = libc::api::stat {
		st_mode: 0,
		st_size: 0,
	};
	let ret = unsafe { libc::api::fstat(file, &mut stat) };
	if ret < 0 {
		return;
	}

	let mut buf = vec![0u8; stat.st_size as usize];
	let len = read(file, buf.as_mut_ptr() as *mut c_void, buf.len());

	let contents = unsafe {
		let slice = slice::from_raw_parts(buf.as_ptr(), len as usize);
		str::from_utf8_unchecked(slice)
	};
	write(
		STDOUT_FILENO,
		contents.as_ptr() as *const c_void,
		contents.len(),
	);
}

fn cd(path: Option<&str>) {
	let path = path
		.map(|rstr| CString::new(rstr).unwrap())
		.unwrap_or(CString::new("/").unwrap());
	chdir(path.as_ptr());
}

fn uptime() {
	let time = format!("{}\n", syscall::uptime());
	write(STDOUT_FILENO, time.as_ptr() as *const c_void, time.len());
}

fn ls(path: Option<&str>) {
	let path = path
		.map(|rstr| CString::new(rstr).unwrap())
		.unwrap_or(CString::new(".").unwrap());
	let fd_root = opendir(path.as_ptr());

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

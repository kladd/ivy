use alloc::vec;
use core::{cmp::min, fmt::Write};

use log::trace;

use crate::{
	arch::x86::enable_interrupts, fs::file_descriptor::FileDescriptor,
	proc::Task,
};
use crate::{
	arch::x86::{clock::uptime_seconds, halt},
	// ed::ed_main,
	time::DateTime,
};

pub fn main() {
	enable_interrupts();
	#[cfg(not(feature = "headless"))]
	let mut terminal = FileDescriptor::open("/DEV/CONSOLE").unwrap();
	#[cfg(feature = "headless")]
	let mut terminal = FileDescriptor::open("/DEV/SERIAL").unwrap();

	loop {
		terminal.write_str("@ ").unwrap();

		let line = terminal.read_line();
		let mut tokens = line.split_ascii_whitespace();

		match tokens.next() {
			Some("ls") => ls_main(tokens.next()),
			Some("cat") => {
				if let Some(path) = tokens.next() {
					cat_main(path)
				}
			}
			Some("cd") => {
				if let Some(dir) = tokens.next().and_then(FileDescriptor::open)
				{
					unsafe { (*Task::current()).chdir(&dir) }
				}
			}
			// Some("ed") => ed_main(),
			Some("uptime") => {
				terminal
					.write_fmt(format_args!("{}\n", uptime_seconds()))
					.unwrap();
			}
			Some("date") => terminal
				.write_fmt(format_args!("{}\n", DateTime::now()))
				.unwrap(),
			Some("touch") => {
				todo!();
				// tokens
				// 	.next()
				// 	.filter(|fname| FileDescriptor::open(fname).is_none())
				// 	.map(/* create */);
			}
			_ => {
				trace!("continuing");
				continue;
			}
		}
		halt();
	}
}

fn ls_main(path: Option<&str>) {
	let dir = match path {
		Some(p) => {
			if let Some(f) = FileDescriptor::open(p) {
				f
			} else {
				return;
			}
		}
		None => FileDescriptor::open(".").unwrap(),
	};

	#[cfg(not(feature = "headless"))]
	let mut terminal = FileDescriptor::open("/DEV/CONSOLE").unwrap();
	#[cfg(feature = "headless")]
	let mut terminal = FileDescriptor::open("/DEV/SERIAL").unwrap();

	for node in dir.readdir().iter() {
		if node.is_dir() {
			terminal
				.write_fmt(format_args!(
					"    {:5} {:8} {:12}\n",
					"<DIR>",
					"",
					node.name()
				))
				.unwrap();
		} else {
			terminal
				.write_fmt(format_args!(
					"    {:5} {:8} {:12}\n",
					"",
					node.size(),
					node.name()
				))
				.unwrap();
		}
	}
}

fn cat_main(path: &str) {
	#[cfg(not(feature = "headless"))]
	let mut terminal = FileDescriptor::open("/DEV/CONSOLE").unwrap();
	#[cfg(feature = "headless")]
	let mut terminal = FileDescriptor::open("/DEV/SERIAL").unwrap();
	match FileDescriptor::open(path) {
		None => (),
		Some(fd) if fd.is_dir() => {
			terminal
				.write_fmt(format_args!("cat: {}: Is a directory", fd.name()))
				.unwrap();
		}
		Some(fd) => {
			let mut buf = vec![0; min(512, fd.size())];
			let n = fd.read(&mut buf);
			for i in 0..n {
				terminal.write_char(buf[i] as char).unwrap()
			}
		}
	}
}

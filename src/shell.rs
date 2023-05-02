use alloc::{format, vec};
use core::{cmp::min, fmt::Write};

use log::trace;

use crate::{
	arch::x86::{clock::uptime_seconds, halt},
	ed::ed_main,
	fs::{create, open, read, readdir, write, File},
	proc::Task,
	time::DateTime,
};

pub fn main() {
	let File::Terminal(mut terminal) = open("CONS").unwrap() else {
		panic!("Failed to open console device")
	};

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
				if let Some(dir) = tokens.next().and_then(open) {
					unsafe { (*Task::current()).chdir(&dir) }
				}
			}
			Some("ed") => ed_main(),
			Some("uptime") => {
				terminal
					.write_fmt(format_args!("{}\n", uptime_seconds()))
					.unwrap();
			}
			Some("date") => terminal
				.write_fmt(format_args!("{}\n", DateTime::now()))
				.unwrap(),
			Some("touch") => {
				tokens
					.next()
					.filter(|fname| open(fname).is_none())
					.map(create);
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
	let mut term = open("CONS").unwrap();

	let mut dir = match path {
		Some(p) => {
			if let Some(f) = open(p) {
				f
			} else {
				return;
			}
		}
		None => open(".").unwrap(),
	};

	if let File::File(file) = dir {
		write(
			&mut term,
			format!("    {:5} {:8} {:12}\n", "", file.size(), file.name(),)
				.as_bytes(),
		);
		return;
	}

	while let Some(de) = readdir(&mut dir) {
		// Ignore LFN entries for now.
		if !de.is_normal() {
			continue;
		}

		if de.is_dir() {
			write(
				&mut term,
				format!("    {:5} {:8} {:12}\n", "<DIR>", "", de.name())
					.as_bytes(),
			);
		} else {
			write(
				&mut term,
				format!("    {:5} {:8} {:12}\n", "", de.size(), de.name())
					.as_bytes(),
			);
		}
	}
}

fn cat_main(path: &str) {
	match open(path) {
		Some(File::File(f)) => {
			let mut term = open("CONS").unwrap();
			let mut buf = vec![0; min(512, f.size())];

			read(&mut File::File(f), &mut buf);
			write(&mut term, &buf);
		}
		Some(_) | None => (),
	}
}

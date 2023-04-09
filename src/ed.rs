use core::fmt::Write;

use crate::{
	fs::{Directory, FATFileSystem},
	std::{io::Terminal, string::String, vec::Vec},
};

#[derive(Debug)]
enum Mode {
	Command,
	Append,
}

pub fn ed_main(term: &mut Terminal, fs: &FATFileSystem, cwd: &Directory) {
	let mut mode = Mode::Command;
	let mut line_buf = Vec::new(16);

	loop {
		let line = kdbg!(term.read_line());

		match mode {
			Mode::Command => match line.as_ref().trim() {
				"a" => mode = kdbg!(Mode::Append),
				"w" => term
					.write_fmt(format_args!("{}\n", write_file(&line_buf)))
					.unwrap(),
				"q" => break,
				"p" => display_file(&line_buf, term),
				_ => continue,
			},
			Mode::Append => match line.as_ref() {
				"." => mode = kdbg!(Mode::Command),
				_ => line_buf.push(line),
			},
		}
	}
}

fn display_file(buf: &Vec<String>, term: &mut Terminal) {
	buf.iter()
		.for_each(|s| term.write_fmt(format_args!("{}\n", s)).unwrap())
}

fn write_file(buf: &Vec<String>) -> usize {
	buf.iter().map(|s| s.len()).sum()
}

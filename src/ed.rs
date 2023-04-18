use alloc::{format, string::String, vec::Vec};
use core::fmt::Write;

use crate::{
	fat::{Directory, DirectoryEntry, FATFileSystem},
	std::io::Terminal,
};

#[derive(Debug)]
enum Mode {
	Command,
	Append,
}

pub fn ed_main(term: &mut Terminal, fs: &FATFileSystem, cwd: &mut Directory) {
	let mut mode = Mode::Command;
	let mut line_buf = Vec::with_capacity(16);

	loop {
		let line = kdbg!(term.read_line());
		let mut tokens = line.split_ascii_whitespace();

		match mode {
			Mode::Command => match tokens.next().unwrap().trim() {
				"a" => mode = kdbg!(Mode::Append),
				"w" => {
					let name = tokens.next().unwrap().trim();
					let size = file_size(&line_buf);

					let mut data = Vec::with_capacity(size);
					for line in line_buf.iter() {
						for byte in line.as_bytes() {
							data.push(*byte);
						}
					}

					let mut file = DirectoryEntry::new(name);
					fs.write_file(&mut file, &data);
					cwd.add_entry(file);

					term.write_fmt(format_args!("{size}\n")).unwrap();
				}
				"q" => break,
				"p" => display_file(&line_buf, term),
				_ => continue,
			},
			Mode::Append => match line.as_ref() {
				"." => mode = kdbg!(Mode::Command),
				_ => line_buf.push(format!("{line}\n")),
			},
		}
	}
}

fn display_file(buf: &Vec<String>, term: &mut Terminal) {
	buf.iter().for_each(|s| term.write_str(s).unwrap())
}

fn file_size(buf: &Vec<String>) -> usize {
	buf.iter().map(|s| s.len()).sum()
}

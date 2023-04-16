use core::fmt::Write;

use crate::{
	fat::{Directory, DirectoryEntry, FATFileSystem},
	std::{io::Terminal, string::String, vec::Vec},
};

#[derive(Debug)]
enum Mode {
	Command,
	Append,
}

pub fn ed_main(term: &mut Terminal, fs: &FATFileSystem, cwd: &mut Directory) {
	let mut mode = Mode::Command;
	let mut line_buf = Vec::new(16);

	loop {
		let line = kdbg!(term.read_line());
		let mut tokens = line.split_ascii_whitespace();

		match mode {
			Mode::Command => match tokens.next().unwrap().trim() {
				"a" => mode = kdbg!(Mode::Append),
				"w" => {
					let name = tokens.next().unwrap().trim();
					let size = file_size(&line_buf);

					let mut data = Vec::new(size);
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
				_ => {
					let mut l = String::new(line.len() + 1);
					l.write_fmt(format_args!("{line}\n")).unwrap();
					line_buf.push(l);
				}
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

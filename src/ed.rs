use alloc::{format, string::String, vec::Vec};
use core::fmt::Write;

use crate::fs::{create, open, read_line, write, File};

#[derive(Debug)]
enum Mode {
	Command,
	Append,
}

pub fn ed_main() {
	let mut mode = Mode::Command;
	let mut line_buf = Vec::with_capacity(16);

	let mut term = open("CONS").unwrap();

	loop {
		let line = kdbg!(read_line(&mut term));
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

					let mut file = open(name).unwrap_or_else(|| create(name));

					write(&mut file, &data);
					write(&mut term, format!("{size}\n").as_bytes());
				}
				"q" => break,
				"p" => display_file(&line_buf, &mut term),
				_ => continue,
			},
			Mode::Append => match line.as_ref() {
				"." => mode = kdbg!(Mode::Command),
				_ => line_buf.push(format!("{line}\n")),
			},
		}
	}
}

fn display_file(buf: &Vec<String>, term: &mut File) {
	buf.iter().for_each(|s| {
		write(term, s.as_bytes());
	})
}

fn file_size(buf: &Vec<String>) -> usize {
	buf.iter().map(|s| s.len()).sum()
}

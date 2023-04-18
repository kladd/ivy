use alloc::string::String;
use core::fmt::Write;

use crate::{
	arch::x86::{clock::uptime_seconds, halt},
	ed::ed_main,
	fat::{Directory, DirectoryEntry, FATFileSystem},
	fs::FileSystem,
	keyboard::KBD,
	std::io::Terminal,
	time::DateTime,
	vga::VideoMemory,
};

pub fn main() {
	let fs = FATFileSystem::open(0);
	let mut cwd = fs.root();
	let mut terminal = Terminal {
		kbd: unsafe { &mut KBD },
		vga: VideoMemory::get(),
	};

	loop {
		terminal.write_str("@ ").unwrap();

		let line = terminal.read_line();
		let mut tokens = line.split_ascii_whitespace();

		match tokens.next() {
			Some("ls") => {
				let dir_maybe = tokens
					.next()
					.and_then(|path| fs.find_path(Some(cwd), path));

				ls_main(&mut terminal, &fs, &dir_maybe.unwrap_or(cwd));
			}
			Some("cat") => tokens
				.next()
				.and_then(|file_name| fs.find_path(Some(cwd), file_name))
				.map(|entry| {
					cat_main(&mut terminal, &fs, &entry);
				})
				.unwrap(),
			Some("cd") => {
				if let Some(dir) = tokens
					.next()
					.and_then(|dir_name| fs.find_path(Some(cwd), dir_name))
				{
					cwd = dir;
				}
			}
			// TODO: use file pointers instead of `as_dir()`
			Some("ed") => ed_main(&mut terminal, &fs, &mut cwd.as_dir(&fs)),
			Some("uptime") => {
				terminal
					.write_fmt(format_args!("{}\n", uptime_seconds()))
					.unwrap();
			}
			Some("date") => terminal
				.write_fmt(format_args!("{}\n", DateTime::now()))
				.unwrap(),
			_ => {
				kprintf!("continuing");
				continue;
			}
		}
		halt();
	}
}

fn ls_main(term: &mut Terminal, fs: &FATFileSystem, dir: &DirectoryEntry) {
	// TODO: ls normal files.
	for entry in dir.as_dir(fs).entries() {
		if !entry.is_normal() {
			continue;
		}
		if entry.is_dir() {
			term.write_fmt(format_args!(
				"    {:5} {:8} {:12}\n",
				"<DIR>",
				"",
				entry.name()
			))
			.unwrap();
		} else {
			term.write_fmt(format_args!(
				"    {:5} {:8} {:12}\n",
				"",
				entry.size(),
				entry.name(),
			))
			.unwrap();
		}
	}
}

fn cat_main(term: &mut Terminal, fs: &FATFileSystem, entry: &DirectoryEntry) {
	let mut bytes = fs.read_file(entry);
	let str = unsafe {
		String::from_raw_parts(bytes.as_mut_ptr(), bytes.len(), bytes.len())
	};
	write!(term, "{str}").unwrap();
}

fn find<'a>(dir: &'a Directory, name: &str) -> Option<&'a DirectoryEntry> {
	dir.entries().iter().find(|entry| entry.name() == name)
}

fn find_dir(
	fs: &FATFileSystem,
	dir: &Directory,
	name: &str,
) -> Option<Directory> {
	find(dir, name).map(|entry| entry.as_dir(fs))
}

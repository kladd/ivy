use core::fmt::Write;

use crate::{
	arch::x86::{clock::uptime_seconds, halt},
	ed::ed_main,
	fs::{Directory, DirectoryEntry, FATFileSystem},
	keyboard::KBD,
	std::{io::Terminal, string::String},
	time::DateTime,
	vga::VideoMemory,
};

pub fn main() {
	let fat_fs = FATFileSystem::open(0);
	let mut cwd = fat_fs.read_root();
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
					.and_then(|dir_name| find_dir(&fat_fs, &cwd, dir_name));
				let dir = match dir_maybe {
					Some(ref dir) => dir,
					_ => &cwd,
				};
				ls_main(&mut terminal, dir);
			}
			Some("cat") => tokens
				.next()
				.and_then(|file_name| find(&cwd, file_name))
				.map(|entry| {
					cat_main(&mut terminal, &fat_fs, entry);
				})
				.unwrap(),
			Some("cd") => {
				if let Some(dir) = tokens
					.next()
					.and_then(|dir_name| find_dir(&fat_fs, &cwd, dir_name))
				{
					cwd = dir;
				}
			}
			Some("ed") => ed_main(&mut terminal, &fat_fs, &mut cwd),
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

fn ls_main(term: &mut Terminal, dir: &Directory) {
	kprintf!("ls_main()");
	for entry in dir.entries() {
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
	term.write_str(String::from_ascii_own(fs.read_file(entry)).as_ref())
		.unwrap();
}

fn find<'a>(dir: &'a Directory, name: &str) -> Option<&'a DirectoryEntry> {
	dir.entries()
		.iter()
		.find(|entry| entry.name().as_ref() == name)
}

fn find_dir(
	fs: &FATFileSystem,
	dir: &Directory,
	name: &str,
) -> Option<Directory> {
	find(dir, name).map(|entry| entry.as_dir(fs))
}

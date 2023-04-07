use core::fmt::Write;

use crate::{
	arch::x86::{clock::uptime_seconds, halt},
	fs::{Directory, DirectoryEntry, FATFileSystem},
	keyboard::{Keycode, KBD},
	std::string::String,
	vga::VideoMemory,
};

pub fn shell_main() {
	let mut vga = VideoMemory::get();
	let fat_fs = FATFileSystem::open(0);
	let mut cwd = fat_fs.read_root();

	loop {
		write!(vga, "# ").unwrap();

		let line = read_string();
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
				ls_main(dir)
			}
			Some("cat") => tokens
				.next()
				.and_then(|file_name| find(&cwd, file_name))
				.map(|entry| {
					cat_main(&fat_fs, entry);
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
			Some("uptime") => {
				writeln!(vga, "{}", uptime_seconds()).unwrap();
			}
			_ => {
				kprintf!("continuing");
				continue;
			}
		}
		halt();
	}
}

fn ls_main(dir: &Directory) {
	kprintf!("ls_main()");
	let mut vga = VideoMemory::get();

	writeln!(vga, "\n  Directory of {}\n", dir.name()).unwrap();
	for entry in dir.entries() {
		if entry.is_dir() {
			writeln!(vga, "    {:5} {:8} {:12}", "<DIR>", "", entry.name())
				.unwrap();
		} else {
			writeln!(
				vga,
				"    {:5} {:8} {:12}",
				"",
				entry.size(),
				entry.name(),
			)
			.unwrap();
		}
	}
}

fn cat_main(fs: &FATFileSystem, entry: &DirectoryEntry) {
	writeln!(
		VideoMemory::get(),
		"{}",
		String::from_ascii_own(fs.read_file(entry))
	)
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

fn read_string() -> String {
	let mut vga = VideoMemory::get();
	let mut s = String::new(80);

	loop {
		match unsafe { KBD.getc() } {
			Some(Keycode::Newline) => {
				vga.insert_newline().unwrap();
				return kdbg!(s);
			}
			Some(Keycode::Char(c)) => {
				vga.write_char(c).unwrap();
				s.write_char(c).unwrap();
			}
			Some(Keycode::FormFeed) => vga.form_feed(),
			Some(Keycode::VerticalTab) => vga.vertical_tab(),
			Some(Keycode::Nak) => vga.nak(),
			Some(Keycode::Backspace) => {
				s.pop();
				vga.backspace();
			}
			_ => (),
		};
		halt();
	}
}

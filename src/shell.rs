use alloc::{string::String, vec};
use core::{cmp::min, fmt::Write, mem::size_of};

use crate::{
	arch::x86::{clock::uptime_seconds, halt},
	ed::ed_main,
	fat::{DirectoryEntry, DirectoryEntryNode, FATFileSystem},
	keyboard::KBD,
	std::io::Terminal,
	time::DateTime,
	vga::VideoMemory,
};

pub fn main() {
	let fs = FATFileSystem::new(0);
	let mut cwd = fs.find(&fs.root(), "/HOME/USER").unwrap();
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
				match tokens.next().and_then(|path| fs.find(&cwd, path)) {
					Some(mut node) => ls_main(&mut terminal, &fs, &mut node),
					_ => ls_main(&mut terminal, &fs, &mut cwd),
				};
			}
			Some("cat") => {
				let file_maybe = tokens
					.next()
					.and_then(|file_name| fs.find(&cwd, file_name));
				match file_maybe {
					Some(mut f) => cat_main(&mut terminal, &fs, &mut f),
					None => continue,
				};
			}
			Some("cd") => {
				if let Some(dir) =
					tokens.next().and_then(|dir_name| fs.find(&cwd, dir_name))
				{
					cwd = dir;
				}
			}
			Some("ed") => ed_main(&mut terminal, &fs, cwd),
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
					.filter(|fname| fs.find(&cwd, fname).is_none())
					.map(|fname| fs.create(&mut cwd, fname));
			}
			_ => {
				kprintf!("continuing");
				continue;
			}
		}
		halt();
	}
}

fn ls_main(
	term: &mut Terminal,
	fs: &FATFileSystem,
	node: &mut DirectoryEntryNode,
) {
	if !node.entry.is_dir() {
		term.write_fmt(format_args!(
			"    {:5} {:8} {:12}\n",
			"",
			node.entry.size(),
			node.entry.name(),
		))
		.unwrap();
		return;
	}

	let mut dir = fs.open(node);

	let mut buf = [0u8; size_of::<DirectoryEntry>()];

	// The first entry is this directory, consume it.
	dir.read(&mut buf);

	// Now list the contents of this directory.
	while dir.read(&mut buf) != 0 {
		// Listing is complete when byte[0] is 0.
		if buf[0] == 0 {
			break;
		}

		let de: DirectoryEntry =
			unsafe { *(&buf as *const u8 as *const DirectoryEntry) };

		// Ignore LFN entries for now.
		if !de.is_normal() {
			continue;
		}

		if de.is_dir() {
			term.write_fmt(format_args!(
				"    {:5} {:8} {:12}\n",
				"<DIR>",
				"",
				de.name()
			))
			.unwrap();
		} else {
			term.write_fmt(format_args!(
				"    {:5} {:8} {:12}\n",
				"",
				de.size(),
				de.name(),
			))
			.unwrap();
		}
	}
}

fn cat_main(
	term: &mut Terminal,
	fs: &FATFileSystem,
	node: &mut DirectoryEntryNode,
) {
	let size = node.entry.size() as usize;
	let mut f = fs.open(node);
	let mut buf = vec![0; min(512, size)];

	f.read(&mut buf);

	write!(term, "{}", unsafe { String::from_utf8_unchecked(buf) }).unwrap();
}

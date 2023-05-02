use alloc::string::String;
use core::{cmp::min, fmt::Write, mem::size_of, slice};

use crate::{
	fat, fat::DirectoryEntry, keyboard::KBD, proc::Task, std::io,
	vga::VideoMemory,
};

pub enum File<'a> {
	Directory(fat::File<'a>),
	File(fat::File<'a>),
	Terminal(io::Terminal<'a>),
}

pub fn create(path: &str) -> File {
	let task = unsafe { &*Task::current() };
	let fs = task.fs;

	File::File(fs.open(fs.create(task.cwd, path)))
}

pub fn open(path: &str) -> Option<File> {
	Some(match path {
		"." => {
			// FAT16 doesn't have '.' for root.
			let task = unsafe { &*Task::current() };
			let fs = task.fs;
			File::Directory(fs.open(*task.cwd))
		}
		"CONS" => File::Terminal(io::Terminal {
			kbd: unsafe { &mut KBD },
			vga: VideoMemory::get(),
		}),
		_ => {
			let task = unsafe { &*Task::current() };
			let fs = task.fs;

			let node = fs.find(task.cwd, path)?;

			let f = fs.open(node);
			if node.entry.is_dir() {
				File::Directory(f)
			} else {
				File::File(f)
			}
		}
	})
}

pub fn read(f: &mut File, buf: &mut [u8]) -> usize {
	match f {
		File::Directory(f) | File::File(f) => f.read(buf),
		File::Terminal(t) => {
			let s = t.read_line();
			let n = min(buf.len(), s.len());
			buf[..n].copy_from_slice(&s.as_bytes()[..n]);
			n
		}
	}
}

pub fn readdir(f: &mut File) -> Option<DirectoryEntry> {
	let dir_entry_size = size_of::<DirectoryEntry>();

	let dir_entry = DirectoryEntry::default();
	let dir_entry_buf = unsafe {
		slice::from_raw_parts_mut(
			&dir_entry as *const _ as *mut u8,
			dir_entry_size,
		)
	};

	assert_eq!(read(f, dir_entry_buf), dir_entry_size);

	dir_entry.present()
}

pub fn read_line(f: &mut File) -> String {
	if let File::Terminal(t) = f {
		t.read_line()
	} else {
		unimplemented!()
	}
}

pub fn write(f: &mut File, buf: &[u8]) -> usize {
	match f {
		File::Directory(f) | File::File(f) => f.write(buf),
		File::Terminal(t) => {
			t.write_str(unsafe { core::str::from_utf8_unchecked(buf) })
				.unwrap();
			buf.len()
		}
	}
}

pub fn seek(f: &mut File, offset: usize) {
	match f {
		File::Directory(f) | File::File(f) => f.seek(offset),
		_ => (),
	}
}

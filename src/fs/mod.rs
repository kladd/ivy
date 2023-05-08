pub mod dev;
pub mod fat;
pub mod file_descriptor;
pub mod inode;

use alloc::{boxed::Box, vec::Vec};
use core::{
	ptr::null_mut,
	sync::atomic::{AtomicPtr, Ordering},
};

use crate::fs::{
	file_descriptor::FileDescriptor,
	inode::{Inode, InodeHash},
};

#[derive(Debug)]
pub struct MountPoint {
	host_inode_hash: Option<InodeHash>,
	guest_root_inode: Inode,
}

#[derive(Debug)]
pub struct FileSystem {
	mounts: Vec<MountPoint>,
}

impl FileSystem {
	pub fn root(&self) -> &Inode {
		&self.mounts[0].guest_root_inode
	}

	// TODO: Result, not option.
	pub fn open(&self, base: &Inode, path: &str) -> Option<FileDescriptor> {
		self.find(base, path).map(FileDescriptor::from)
	}

	pub fn mount_root(&mut self, root: Inode) {
		assert!(self.mounts.is_empty(), "Root filesystem already mounted");
		self.mount_inode(None, root);
	}

	pub fn mount(&mut self, path: &str, guest_root: Inode) {
		// TODO: Handle mount points that don't exist.
		// TODO: Mount with arbitrary relative paths instead of relative to
		//       `self.root()`.
		let host_inode = self
			.find(self.root(), path)
			.expect("Mount point doesn't exist");
		self.mount_inode(Some(host_inode), guest_root);
	}

	pub fn find(&self, base: &Inode, path: &str) -> Option<Inode> {
		if path == "." {
			Some(base.clone())
		} else if path.starts_with("/") {
			self.find(self.root(), &path[1..])
		} else {
			let segments = path.split("/");
			let mut node = base.clone();

			for segment in segments {
				if segment.is_empty() {
					continue;
				}

				node = node.lookup(segment)?;

				// If this node is a mount point, start traversing that mounted
				// filesystem by returning its root here.
				if let Some(mp) = self.mount_point(&node) {
					node = mp
				}
			}

			Some(node)
		}
	}
}

impl FileSystem {
	fn mount_inode(&mut self, host_inode: Option<Inode>, guest_root: Inode) {
		self.mounts.push(MountPoint {
			host_inode_hash: host_inode.map(|inode| inode.hash()),
			guest_root_inode: guest_root,
		});
	}

	fn mount_point(&self, node: &Inode) -> Option<Inode> {
		self.mounts
			.iter()
			.find(|mp| {
				if let Some(hash) = &mp.host_inode_hash {
					*hash == node.hash()
				} else {
					false
				}
			})
			.map(|mp| mp.guest_root_inode.clone())
	}
}

// static mut GLOBAL: MaybeUninit<Pin<Box<FileSystem>>> = MaybeUninit::uninit();

static GLOBAL: AtomicPtr<FileSystem> = AtomicPtr::new(null_mut());

impl FileSystem {
	pub fn init() {
		let fs = Box::new(Self { mounts: Vec::new() });
		GLOBAL
			.compare_exchange(
				null_mut(),
				Box::leak(fs),
				Ordering::Acquire,
				Ordering::Relaxed,
			)
			.unwrap();
	}

	pub fn current() -> &'static mut FileSystem {
		unsafe { &mut *GLOBAL.load(Ordering::Relaxed) }
	}
}

// pub fn create(path: &str) -> File {
// 	let task = unsafe { &*Task::current() };
// 	let fs = task.fs;
//
// 	File::File(fs.open(fs.create(task.cwd, path)))
// }

// pub fn open(path: &str) -> Option<File> {
// 	Some(match path {
// 		"." => {
// 			// FAT16 doesn't have '.' for root.
// 			let task = unsafe { &*Task::current() };
// 			let fs = task.fs;
// 			File::Directory(fs.open(*task.cwd))
// 		}
// 		#[cfg(not(feature = "headless"))]
// 		"CONS" => File::Terminal(io::Terminal {
// 			read: unsafe { &mut KBD },
// 			write: unsafe { &mut VGA },
// 		}),
// 		#[cfg(feature = "headless")]
// 		"CONS" => File::Terminal(io::Terminal {
// 			read: unsafe { &mut COM1 },
// 			write: unsafe { &mut COM1 },
// 		}),
// 		_ => {
// 			let task = unsafe { &*Task::current() };
// 			let fs = task.fs;
//
// 			let node = fs.find(task.cwd, path)?;
//
// 			let f = fs.open(node);
// 			if node.entry.is_dir() {
// 				File::Directory(f)
// 			} else {
// 				File::File(f)
// 			}
// 		}
// 	})
// }

// pub fn read(f: &mut File, buf: &mut [u8]) -> usize {
// 	match f {
// 		File::Directory(f) | File::File(f) => f.read(buf),
// 		File::Terminal(t) => {
// 			let s = t.read_line();
// 			let n = min(buf.len(), s.len());
// 			buf[..n].copy_from_slice(&s.as_bytes()[..n]);
// 			n
// 		}
// 	}
// }
//
// pub fn readdir(f: &mut File) -> Option<DirectoryEntry> {
// 	let dir_entry_size = size_of::<DirectoryEntry>();
//
// 	let dir_entry = DirectoryEntry::default();
// 	let dir_entry_buf = unsafe {
// 		slice::from_raw_parts_mut(
// 			&dir_entry as *const _ as *mut u8,
// 			dir_entry_size,
// 		)
// 	};
//
// 	assert_eq!(read(f, dir_entry_buf), dir_entry_size);
//
// 	dir_entry.present()
// }
//
// pub fn read_line(f: &mut File) -> String {
// 	if let File::Terminal(t) = f {
// 		t.read_line()
// 	} else {
// 		unimplemented!()
// 	}
// }
//
// pub fn write(f: &mut File, buf: &[u8]) -> usize {
// 	match f {
// 		File::Directory(f) | File::File(f) => f.write(buf),
// 		File::Terminal(t) => {
// 			t.write_str(unsafe { core::str::from_utf8_unchecked(buf) })
// 				.unwrap();
// 			buf.len()
// 		}
// 	}
// }
//
// pub fn seek(f: &mut File, offset: usize) {
// 	match f {
// 		File::Directory(f) | File::File(f) => f.seek(offset),
// 		_ => (),
// 	}
// }

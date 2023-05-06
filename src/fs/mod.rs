pub mod dev;
pub mod fat;
pub mod inode;

use alloc::{vec, vec::Vec};

use crate::fs::inode::{Inode, InodeHash};

#[derive(Debug)]
pub struct FileDescriptor {
	offset: usize,
	inode: Inode,
}

pub struct MountPoint {
	host_inode_hash: Option<InodeHash>,
	guest_root_inode: Inode,
}

pub struct FileSystem {
	mounts: Vec<MountPoint>,
}

impl FileSystem {
	pub fn new(root_inode: Inode) -> Self {
		Self {
			mounts: vec![MountPoint {
				host_inode_hash: None,
				guest_root_inode: root_inode,
			}],
		}
	}

	pub fn root(&self) -> &Inode {
		&self.mounts[0].guest_root_inode
	}

	// TODO: Result, not option.
	pub fn open(&self, base: &Inode, path: &str) -> Option<FileDescriptor> {
		self.find(base, path).map(FileDescriptor::from)
	}

	pub fn mount(&mut self, path: &str, guest_root: Inode) {
		// TODO: Handle mount points that don't exist.
		// TODO: Mount with arbitrary relative paths instead of relative to
		//       `self.root()`.
		let host_inode = self
			.find(self.root(), path)
			.expect("Mount point doesn't exist");
		self.mounts.push(MountPoint {
			host_inode_hash: Some(host_inode.hash()),
			guest_root_inode: guest_root,
		});
	}
}

impl FileSystem {
	fn find(&self, base: &Inode, path: &str) -> Option<Inode> {
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

impl From<Inode> for FileDescriptor {
	fn from(inode: Inode) -> Self {
		Self { offset: 0, inode }
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

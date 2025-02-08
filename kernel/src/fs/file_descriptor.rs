use core::{cmp::min, ffi::c_char, fmt::Write, ptr, slice, str};


use crate::fs::inode::Inode;

#[derive(Debug, Clone)]
pub struct FileDescriptor {
	pub(super) offset: usize,
	pub inode: Inode,
}

impl FileDescriptor {
	pub fn new(inode: Inode) -> Self {
		Self { offset: 0, inode }
	}

	pub fn read(&mut self, dst: *mut u8, len: usize) -> usize {
		assert!(
			self.offset + len <= 0x1000,
			"TODO: Read more than one block"
		);
		let len = match &self.inode {
			Inode::Ext2(inode) => inode.read(self.offset, dst, len),
			Inode::Device(inode) => {
				let line = inode.read_line();
				let len = min(len, line.len());
				unsafe { ptr::copy_nonoverlapping(line.as_ptr(), dst, len) };
				len
			}
		};

		self.offset += len;

		len
	}

	pub fn readdir(&mut self, dst: *mut libc::api::dirent) {
		let dirent = unsafe { &mut *dst };

		match &self.inode {
			Inode::Ext2(inode) => {
				inode.readdir().get(self.offset).map(|de| {
					dirent.d_ino = de.header.inode as u64;
					for (i, c) in de.name.bytes().enumerate() {
						dirent.d_name[i] = c as c_char;
					}
				});
			}
			Inode::Device(inode) => {
				inode.readdir().get(self.offset).map(|devnode| {
					dirent.d_ino = 1;
					for (i, c) in devnode.name().bytes().enumerate() {
						dirent.d_name[i] = c as c_char;
					}
				});
			}
		}

		self.offset += 1;
	}

	pub fn write(&mut self, src: *const u8, len: usize) -> usize {
		match &mut self.inode {
			Inode::Ext2(_) => todo!(),
			Inode::Device(inode) => {
				let s = unsafe {
					str::from_utf8_unchecked(slice::from_raw_parts(src, len))
				};
				inode.write_str(s).expect("failed to write to dev");
			}
		}

		self.offset += len;

		len
	}
}

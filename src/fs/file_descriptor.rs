use alloc::{string::String, vec::Vec};
use core::fmt::Write;

use crate::{
	fs::{inode::Inode, FileSystem},
	proc::Task,
};

#[derive(Debug)]
pub struct FileDescriptor {
	pub(super) offset: usize,
	pub inode: Inode,
}

impl FileDescriptor {
	pub fn open(path: &str) -> Option<Self> {
		let task = unsafe { &*Task::current() };
		let fs = unsafe { &*FileSystem::current() };

		Some(Self {
			offset: 0,
			inode: fs.find(&task.cwd, path)?,
		})
	}

	pub fn read(&self, buf: &mut [u8]) -> usize {
		// TODO: Read all blocks.
		assert!(self.offset + buf.len() <= 512);

		match &self.inode {
			Inode::FAT(inode) => inode.read(self.offset, buf),
			Inode::Device(_) => todo!(),
		}
	}

	pub fn readdir(&self) -> Vec<Inode> {
		match &self.inode {
			Inode::FAT(inode) => inode.readdir(),
			Inode::Device(inode) => inode.readdir(),
		}
	}

	pub fn read_line(&self) -> String {
		match &self.inode {
			Inode::Device(inode) => inode.read_line(),
			_ => todo!(),
		}
	}
}

// TODO: `stat`
impl FileDescriptor {
	pub fn is_dir(&self) -> bool {
		self.inode.is_dir()
	}

	pub fn size(&self) -> usize {
		self.inode.size()
	}

	pub fn name(&self) -> String {
		self.inode.name()
	}
}

impl Write for FileDescriptor {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		match self.inode {
			Inode::Device(ref mut inode) => inode.write_str(s),
			_ => unimplemented!(),
		}
	}
}

impl From<Inode> for FileDescriptor {
	fn from(inode: Inode) -> Self {
		Self { offset: 0, inode }
	}
}

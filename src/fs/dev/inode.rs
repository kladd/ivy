use crate::fs::inode::{Inode, InodeHash};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeviceInode {
	Root,
	Console,
	Serial,
}

impl DeviceInode {
	pub fn lookup(&self, name: &str) -> Option<Inode> {
		match name {
			"CONSOLE" => Some(Inode::Device(DeviceInode::Console)),
			"SERIAL" => Some(Inode::Device(DeviceInode::Serial)),
			_ => None,
		}
	}

	pub fn hash(&self) -> InodeHash {
		InodeHash::Device(*self)
	}
}

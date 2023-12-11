use alloc::string::String;
use core::mem;

use crate::fs::device::inode::DeviceInode;

#[derive(Debug, Copy, Clone)]
pub enum Inode {
	Device(DeviceInode),
}

#[derive(PartialEq, Debug)]
pub enum InodeHash {
	Device(mem::Discriminant<DeviceInode>),
}

// TODO: When this API is stable, make the Inode subtypes implement a trait so
//       we're not switching on this enum all the time. Not that it's that bad.
impl Inode {
	pub fn lookup(&self, name: &str) -> Option<Self> {
		match self {
			Self::Device(node) => Some(node.lookup(name)?),
		}
	}

	pub fn hash(&self) -> InodeHash {
		match self {
			Self::Device(node) => node.hash(),
		}
	}
}

// TODO: `stat`
impl Inode {
	pub fn name(&self) -> String {
		match self {
			Inode::Device(inode) => inode.name(),
		}
	}

	pub fn size(&self) -> usize {
		match self {
			Inode::Device(inode) => inode.size(),
		}
	}

	pub fn is_dir(&self) -> bool {
		match self {
			Inode::Device(inode) => inode.is_dir(),
		}
	}
}

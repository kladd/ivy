use crate::fs::{dev::inode::DeviceInode, fat::inode::FATInode};

#[derive(Debug, Clone)]
pub enum Inode {
	FAT(FATInode),
	Device(DeviceInode),
}

#[derive(PartialEq, Debug)]
pub enum InodeHash {
	FAT(u32),
	Device(DeviceInode),
}

impl Inode {
	pub fn lookup(&self, name: &str) -> Option<Self> {
		match self {
			Self::FAT(node) => Some(node.lookup(name)?),
			Self::Device(node) => Some(node.lookup(name)?),
		}
	}

	pub fn hash(&self) -> InodeHash {
		match self {
			Self::FAT(node) => node.hash(),
			Self::Device(node) => node.hash(),
		}
	}
}

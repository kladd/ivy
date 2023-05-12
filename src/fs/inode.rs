use alloc::string::String;
use core::mem;

use crate::fs::{dev::inode::DeviceInode, fat::inode::FATInode};

#[derive(Debug, Copy, Clone)]
pub enum Inode {
	FAT(FATInode),
	Device(DeviceInode),
}

#[derive(PartialEq, Debug)]
pub enum InodeHash {
	FAT(u32),
	Device(mem::Discriminant<DeviceInode>),
}

// TODO: When this API is stable, make the Inode subtypes implement a trait so
//       we're not switching on this enum all the time. Not that it's that bad.
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

// TODO: `stat`
impl Inode {
	pub fn name(&self) -> String {
		match self {
			Inode::FAT(inode) => inode.name(),
			Inode::Device(inode) => inode.name(),
		}
	}

	pub fn size(&self) -> usize {
		match self {
			Inode::FAT(inode) => inode.size(),
			Inode::Device(inode) => inode.size(),
		}
	}

	pub fn is_dir(&self) -> bool {
		match self {
			Inode::FAT(inode) => inode.is_dir(),
			Inode::Device(inode) => inode.is_dir(),
		}
	}
}

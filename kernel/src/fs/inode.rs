use alloc::string::String;
use core::mem;

use crate::fs::{device::inode::DeviceInode, ext2};

#[derive(Debug, Clone)]
pub enum Inode {
	Device(DeviceInode),
	Ext2(ext2::Inode),
}

#[derive(PartialEq, Debug)]
pub enum InodeHash {
	Device(mem::Discriminant<DeviceInode>),
	Ext2(u32),
}

impl Inode {
	pub fn lookup(&self, name: &str) -> Option<Self> {
		match self {
			Self::Device(node) => Some(node.lookup(name)?),
			Inode::Ext2(inode) => Some(Inode::Ext2(inode.lookup(name)?)),
		}
	}

	pub fn hash(&self) -> InodeHash {
		match self {
			Self::Device(node) => node.hash(),
			Inode::Ext2(inode) => InodeHash::Ext2(inode.hash()),
		}
	}
}

impl Inode {
	pub fn name(&self) -> String {
		match self {
			Inode::Device(inode) => inode.name(),
			Inode::Ext2(inode) => inode.name(),
		}
	}

	pub fn is_dir(&self) -> bool {
		match self {
			Inode::Device(inode) => inode.is_dir(),
			Inode::Ext2(inode) => inode.is_dir(),
		}
	}
}

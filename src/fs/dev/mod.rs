use crate::fs::{dev::inode::DeviceInode, inode::Inode};

pub mod inode;

pub struct DeviceFileSystem;

impl DeviceFileSystem {
	pub fn root(&self) -> Inode {
		Inode::Device(DeviceInode::Root)
	}
}

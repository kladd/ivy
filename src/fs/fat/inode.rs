use alloc::rc::Rc;

use crate::{
	arch::x86::ide::{read_offset, read_sector},
	fs::{
		fat::{DirectoryEntry, FATFileSystem},
		inode::{Inode, InodeHash},
	},
};

#[derive(Debug, Clone)]
pub struct FATInode {
	// Retains the address of `entry` in its parent directory for updating
	// attributes, etc.
	pub(super) entry: DirectoryEntry,

	pub(super) dir_sector: u32,
	pub(super) cluster_offset: usize,
	pub(super) fs: Rc<FATFileSystem>,
}

impl FATInode {
	pub fn lookup(&self, name: &str) -> Option<Inode> {
		assert!(self.entry.is_dir());

		read_sector(self.fs.device, self.fs.data_sector_lba(&self.entry));
		let entries = unsafe { read_offset::<[DirectoryEntry; 16]>(0) };

		entries
			.into_iter()
			.enumerate()
			.find(|(_, entry)| entry.name() == name)
			.map(|(cluster_offset, entry)| FATInode {
				fs: self.fs.clone(),
				entry,
				cluster_offset,
				dir_sector: self.entry.first_cluster_lo as u32,
			})
			.map(Inode::FAT)
	}

	pub fn hash(&self) -> InodeHash {
		// TODO: Ignores other FAT devices, ignores cluster_hi
		InodeHash::FAT(self.entry.first_cluster_lo as u32)
	}
}

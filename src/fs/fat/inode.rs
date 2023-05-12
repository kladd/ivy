use alloc::{string::String, vec, vec::Vec};
use core::{cmp::min, mem};

use crate::{
	arch::x86::ide::{read, read_offset, read_sector},
	fs::{
		fat::{DirectoryEntry, FATFileSystem},
		inode::{Inode, InodeHash},
	},
};

#[derive(Debug, Copy, Clone)]
pub struct FATInode {
	// Retains the address of `entry` in its parent directory for updating
	// attributes, etc.
	pub dir_sector: u32,
	pub cluster_offset: usize,
	pub fs: FATFileSystem,

	pub entry: DirectoryEntry,
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

	pub fn read(&self, offset: usize, buf: &mut [u8]) -> usize {
		assert!(!self.entry.is_dir());

		// TODO: Read other blocks/clusters.
		assert!(
			offset + buf.len() < 512,
			"TODO: File reads limited to 512 bytes total"
		);

		// TODO: Block-wise reading rather than sector-based.
		read_sector(self.fs.device, self.fs.data_sector_lba(&self.entry));

		let count = min(self.entry.size() as usize - offset, buf.len());
		read(offset, count, buf);

		count
	}

	pub fn readdir(&self) -> Vec<Inode> {
		if !self.entry.is_dir() {
			return vec![Inode::FAT(self.clone())];
		}

		let mut buffer = [0u8; 512];

		read_sector(self.fs.device, self.fs.data_sector_lba(&self.entry));
		read(0, 512, &mut buffer);

		unsafe { mem::transmute::<[u8; 512], [DirectoryEntry; 16]>(buffer) }
			.iter()
			.enumerate()
			.filter(|(_, de)| !de.is_empty() && de.is_normal())
			.map(|(offset, de)| {
				Inode::FAT(FATInode {
					entry: *de,
					fs: self.fs.clone(),
					dir_sector: self.fs.data_sector_lba(&self.entry),
					cluster_offset: offset,
				})
			})
			.collect::<Vec<Inode>>()
	}

	pub fn hash(&self) -> InodeHash {
		// TODO: Ignores other FAT devices, ignores cluster_hi
		InodeHash::FAT(self.entry.first_cluster_lo as u32)
	}
}

// TODO: All this is probably coming from `stat` later.
impl FATInode {
	pub fn name(&self) -> String {
		self.entry.name()
	}

	pub fn size(&self) -> usize {
		if self.entry.is_dir() {
			0
		} else {
			self.entry.size() as usize
		}
	}

	pub fn is_dir(&self) -> bool {
		self.entry.is_dir()
	}
}

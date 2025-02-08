use alloc::{
	boxed::Box,
	rc::Rc,
	string::{String, ToString},
	sync::Arc,
	vec::Vec,
};
use core::{cmp::min, slice, str};

use log::trace;

use crate::devices::ide;

const ROOT_INODE: u32 = 2;

#[repr(C)]
#[derive(Debug)]
pub struct Superblock {
	s_inodes_count: u32,
	s_blocks_count: u32,
	s_r_blocks_count: u32,
	s_free_blocks_count: u32,
	s_free_inodes_count: u32,
	s_first_data_block: u32,
	s_log_block_size: u32,
	s_log_frag_size: u32,
	s_blocks_per_group: u32,
	s_frags_per_group: u32,
	s_inodes_per_group: u32,
	s_mtime: u32,
	s_wtime: u32,
	s_mnt_count: u16,
	s_max_mnt_count: u16,
	s_magic: u16,
	s_state: u16,
	s_errors: u16,
	s_minor_rev_level: u16,
	s_lastcheck: u32,
	s_checkinterval: u32,
	s_creator_os: u32,
	s_rev_level: u32,
	s_def_resuid: u16,
	s_def_resgid: u16,
	s_first_ino: u32,
	pub s_inode_size: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct BlockGroupDescriptorTable {
	bg_block_bitmap: u32,
	bg_inode_bitmap: u32,
	pub bg_inode_table: u32,
	bg_free_blocks_count: u16,
	bg_free_inodes_count: u16,
	bg_used_dirs_count: u16,
	bg_pad: u16,
	bg_reserved: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct InodeMetadata {
	pub i_mode: u16,
	i_uid: u16,
	pub i_size: u32,
	i_atime: u32,
	i_ctime: u32,
	i_mtime: u32,
	i_dtime: u32,
	i_gid: u16,
	i_links_count: u16,
	i_blocks: u32,
	i_flags: u32,
	i_osd1: u32,
	pub i_block: [u32; 15],
	i_generation: u32,
	i_file_acl: u32,
	i_dir_acl: u32,
	i_faddr: u32,
	i_osd2: [u32; 3],
}

#[derive(Debug, Clone)]
pub struct Inode {
	pub md: InodeMetadata,
	fs: Arc<FileSystem>,
	name: String,
	inumber: u32,
	pub parent: Option<Rc<Inode>>,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct DirectoryEntryHeader {
	pub inode: u32,
	rec_len: u16,
	pub name_len: u8,
	file_type: u8,
}

pub struct DirectoryEntry {
	pub header: DirectoryEntryHeader,
	pub name: String,
}

#[derive(Debug)]
pub struct FileSystem {
	device: u8,
	superblock: Box<Superblock>,
}

impl FileSystem {
	pub fn new(device: u8) -> Self {
		let superblock = ide::read_type(device, 2);
		Self { device, superblock }
	}

	pub fn root(self: &Arc<Self>) -> Rc<Inode> {
		Rc::new(Inode {
			md: self.inode(ROOT_INODE),
			fs: Arc::clone(self),
			name: String::from("/"),
			inumber: ROOT_INODE,
			parent: None,
		})
	}

	pub fn inode(&self, inode: u32) -> InodeMetadata {
		let bgdt = self.block_group_descriptor(self.block_group(inode));

		let inode_table_sector = self.block_to_sector(bgdt.bg_inode_table);
		let inode_table_index = self.inode_index(inode);
		let inode_sector = ((self.superblock.s_inode_size as u32
			* inode_table_index)
			/ ide::SECTOR_SIZE as u32)
			+ inode_table_sector;
		let inode_sector_offset = (self.superblock.s_inode_size as u32
			* inode_table_index)
			% ide::SECTOR_SIZE as u32;

		assert!((self.superblock.s_inode_size as usize) < ide::SECTOR_SIZE);

		*ide::read_offset::<InodeMetadata>(
			self.device,
			inode_sector as u32,
			inode_sector_offset as usize,
		)
		.clone()
	}

	fn block_group_descriptor(
		&self,
		block_group: u32,
	) -> Box<BlockGroupDescriptorTable> {
		let descriptor_block =
			block_group * self.superblock.s_blocks_per_group + 1;
		ide::read_type(self.device, self.block_to_sector(descriptor_block))
	}

	fn block_size(&self) -> usize {
		1024 << self.superblock.s_log_block_size as usize
	}

	fn block_group(&self, inode: u32) -> u32 {
		(inode - 1) / self.superblock.s_inodes_per_group
	}

	fn inode_index(&self, inode: u32) -> u32 {
		(inode - 1) % self.superblock.s_inodes_per_group
	}

	fn block_to_sector(&self, block: u32) -> u32 {
		block * self.block_sector_count() as u32
	}

	fn block_sector_count(&self) -> usize {
		self.block_size() / ide::SECTOR_SIZE
	}
}

impl Inode {
	pub fn is_dir(&self) -> bool {
		self.md.i_mode & 0x4000 != 0
	}

	pub fn readdir(&self) -> Vec<DirectoryEntry> {
		assert!(self.is_dir());

		let mut entries = Vec::new();

		let mut dirs = ide::read_sector_bytes(
			self.fs.device,
			self.fs.block_to_sector(self.md.i_block[0]),
		);

		let len = dirs.len();
		let ptr = dirs.as_mut_ptr();

		let mut offset = 0isize;
		while offset < len as isize {
			let header = unsafe {
				&*(ptr.offset(offset) as *const DirectoryEntryHeader)
			};

			if header.inode == 0 {
				break;
			}

			offset += size_of::<DirectoryEntryHeader>() as isize;

			let name_len = header.name_len;
			let name = unsafe {
				str::from_utf8(slice::from_raw_parts(
					ptr.offset(offset),
					name_len as usize,
				))
				.unwrap()
			};

			// Don't add '.' and '..' entries for root.
			if self.inumber != 2 || (name != "." && name != "..") {
				entries.push(DirectoryEntry {
					header: header.clone(),
					name: name.to_string(),
				});
			}

			offset += name_len as isize;
			if name_len % 4 != 0 {
				offset += 4 - (name_len as isize % 4);
			}
		}

		entries
	}

	pub fn read(&self, offset: usize, dst: *mut u8, len: usize) -> usize {
		assert!(!self.is_dir());
		assert!(offset < 4096, "TODO: read multiple blocks");

		let len = min(self.md.i_size as usize, len);
		ide::read(
			self.fs.device,
			self.fs.block_to_sector(self.md.i_block[0]),
			offset,
			dst,
			len,
		);
		len
	}

	pub fn lookup(self: &Rc<Self>, name: &str) -> Option<Rc<Inode>> {
		if name == ".." {
			return self.parent.clone();
		}

		trace!("lookup({}/{name})", self.name());
		let dirent = self
			.readdir()
			.into_iter()
			.find(|dirent| kdbg!(&dirent.name) == name)?;
		let inode_md = self.fs.inode(dirent.header.inode);

		Some(Rc::new(Inode {
			md: inode_md,
			fs: self.fs.clone(),
			name: dirent.name,
			inumber: dirent.header.inode,
			parent: Some(self.clone()),
		}))
	}

	pub fn hash(&self) -> u32 {
		self.inumber
	}

	pub fn name(&self) -> String {
		self.name.clone()
	}
}

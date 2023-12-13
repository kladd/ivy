use alloc::boxed::Box;

use log::debug;

use crate::devices::ide;

const ROOT_INODE: usize = 2;

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
#[derive(Debug)]
pub struct Inode {
	i_mode: u16,
	i_uid: u16,
	i_size: u32,
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

#[repr(C)]
#[derive(Debug)]
pub struct DirectoryEntry {
	pub inode: u32,
	rec_len: u16,
	pub name_len: u8,
	file_type: u8,
}

#[derive(Debug)]
pub struct FileSystem {
	device: u8,
	superblock: Box<Superblock>,
}

impl FileSystem {
	pub fn new(device: u8) -> Self {
		let superblock = ide::read(device, 2);
		Self { device, superblock }
	}

	pub fn root(&self) -> Box<Inode> {
		self.inode(ROOT_INODE)
	}

	pub fn inode(&self, inode: usize) -> Box<Inode> {
		let bgdt = self.block_group_descriptor(self.block_group(inode));

		let inode_table_sector =
			self.block_to_sector(bgdt.bg_inode_table as usize);
		let inode_table_index = self.inode_index(inode);
		let inode_sector = ((self.superblock.s_inode_size as usize
			* inode_table_index)
			/ ide::SECTOR_SIZE)
			+ inode_table_sector;
		let inode_sector_offset = (self.superblock.s_inode_size as usize
			* inode_table_index)
			% ide::SECTOR_SIZE;

		assert!((self.superblock.s_inode_size as usize) < ide::SECTOR_SIZE);

		ide::read_offset(self.device, inode_sector as u32, inode_sector_offset)
	}

	fn block_group_descriptor(
		&self,
		block_group: usize,
	) -> Box<BlockGroupDescriptorTable> {
		let descriptor_block =
			block_group * self.superblock.s_blocks_per_group as usize + 1;
		ide::read(
			self.device,
			self.block_sector_start(descriptor_block) as u32,
		)
	}

	fn block_size(&self) -> usize {
		1024 << self.superblock.s_log_block_size as usize
	}

	fn block_group(&self, inode: usize) -> usize {
		(inode - 1) / self.superblock.s_inodes_per_group as usize
	}

	fn inode_index(&self, inode: usize) -> usize {
		(inode - 1) % self.superblock.s_inodes_per_group as usize
	}

	fn block_to_sector(&self, block: usize) -> usize {
		block * self.block_size() / ide::SECTOR_SIZE
	}

	fn block_sector_start(&self, block: usize) -> usize {
		block * self.block_sector_count()
	}

	fn block_sector_count(&self) -> usize {
		self.block_size() / ide::SECTOR_SIZE
	}
}

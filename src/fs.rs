use core::fmt::{Debug, Write};

use crate::{
	arch::x86::ide::{read_offset, read_sector},
	std::{string::String, vec::Vec},
};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum FSType {
	FAT12 = 0x01,
	FAT16 = 0x06,
	FAT16LBA = 0x0E,
	FAT32 = 0x0B,
	FAT32LBA = 0x0C,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
#[allow(dead_code)]
struct MBRCHS {
	head: u8,
	sector: u8,
	cylinder: u8,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
#[allow(dead_code)]
struct Partition {
	attributes: u8,
	chs_start: MBRCHS,
	fs_type: FSType,
	chs_end: MBRCHS,
	lba_start: u32,
	lba_len: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
struct BIOSParameterBlock {
	bytes_per_sector: u16,
	sectors_per_cluster: u8,
	reserved_sector_count: u16,
	num_fats: u8,
	root_entry_count: u16,
	total_sectors_16: u16,
	media: u8,
	fat_sz_16: u16,
	sectors_per_track: u16,
	num_heads: u16,
	hidden_sectors: u32,
	total_sectors_32: u32,
	_ignored_0: [u8; 7],
	volume_label: [u8; 11],
	file_system_type: [u8; 8],
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct DirectoryEntry {
	name: [u8; 8],
	ext: [u8; 3],
	attributes: u8,
	_new_technology_reserved: u8,
	c_time_tenth: u8,
	c_time: u16,
	c_date: u16,
	a_date: u16,
	first_cluster_hi: u16,
	m_time: u16,
	m_date: u16,
	first_cluster_lo: u16,
	size: u32,
}

impl DirectoryEntry {
	const ATTR_READ_ONLY: u8 = 0x01;
	const ATTR_HIDDEN: u8 = 0x02;
	const ATTR_SYSTEM: u8 = 0x04;
	const ATTR_VOLUME_ID: u8 = 0x08;
	const ATTR_DIRECTORY: u8 = 0x10;
	const ATTR_ARCHIVE: u8 = 0x20;

	const SIZE: u32 = 32;

	pub fn name(&self) -> String {
		let name = unsafe { core::str::from_utf8_unchecked(&self.name) }
			.split_ascii_whitespace()
			.next()
			.expect("invalid file name");
		let ext = unsafe { core::str::from_utf8_unchecked(&self.ext) }
			.split_ascii_whitespace()
			.next();

		let mut filename = String::new(12);

		write!(filename, "{}", name).expect("write filename");
		if let Some(ext) = ext {
			write!(filename, ".{}", ext).expect("write extension");
		}

		filename
	}

	pub fn is_dir(&self) -> bool {
		self.attributes & DirectoryEntry::ATTR_DIRECTORY != 0
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	fn data_sector(&self, bpb: &BIOSParameterBlock) -> u32 {
		(self.first_cluster_lo as u32 - 2)
			+ bpb.root_dir_sector()
			+ bpb.root_dir_sectors() as u32
	}
}

impl BIOSParameterBlock {
	fn root_dir_sectors(&self) -> u16 {
		((self.root_entry_count * 32) + (self.bytes_per_sector - 1))
			/ self.bytes_per_sector
	}

	fn total_sectors(&self) -> u32 {
		if self.total_sectors_16 != 0 {
			self.total_sectors_16 as u32
		} else {
			self.total_sectors_32
		}
	}

	fn data_sectors(&self) -> u32 {
		self.total_sectors()
			- (self.reserved_sector_count as u32
				// TODO: fat_sz should be fat_sz_32 if FAT32.
				+ (self.num_fats as u32 * self.fat_sz_16 as u32)
				+ self.root_dir_sectors() as u32)
	}

	fn clusters_count(&self) -> u32 {
		self.data_sectors() / self.sectors_per_cluster as u32
	}

	fn root_dir_sector(&self) -> u32 {
		self.reserved_sector_count as u32
			+ (self.num_fats as u32 * self.fat_sz_16 as u32)
	}
}

pub struct Directory {
	entries: Vec<DirectoryEntry>,
}

// None of this is mean to stay.
impl Directory {
	pub fn root(&self) -> &DirectoryEntry {
		self.entries.get(0)
	}

	pub fn each<F>(&self, mut f: F)
	where
		F: FnMut(&DirectoryEntry),
	{
		for i in 1..self.entries.len() {
			f(self.entries.get(i));
		}
	}
}

pub fn list_root() -> Directory {
	// Read MBR.
	read_sector(0);

	// Read first partition table entry.
	let partition_0 = unsafe { read_offset::<Partition>(0x1BE) };
	kdbg!(&partition_0);

	// Read BPB.
	read_sector(partition_0.lba_start);

	let bpb = unsafe { read_offset::<BIOSParameterBlock>(0x0B) };
	kdbg!(bpb);

	let volume_label = core::str::from_utf8(&bpb.volume_label).unwrap();
	kdbg!(volume_label);

	// "16 directories ought to be enough for anybody"
	let mut listing = Directory {
		entries: Vec::new(16),
	};

	read_sector(kdbg!(bpb.root_dir_sector() as u32 + partition_0.lba_start));
	let mut offset = 0;
	loop {
		if unsafe { read_offset::<u8>(offset) } == 0 {
			break;
		}

		// Commit to the next iteration.
		let dir_index = offset;
		offset += DirectoryEntry::SIZE;

		if unsafe { read_offset::<u8>(dir_index + 11) == 0x0F } {
			// Skip long names for now.
			continue;
		}

		let entry = unsafe { read_offset::<DirectoryEntry>(dir_index) };
		kprintf!("Found directory: {}", entry.name());

		listing.entries.push(entry);
	}

	listing
}

use core::fmt::{Debug, Write};

use crate::{
	arch::x86::ide::{read_offset, read_offset_to_vec, read_sector},
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

	pub fn as_dir(&self, fs: &FATFileSystem) -> Directory {
		assert!(self.is_dir());
		Directory::new(self.name(), fs.device, fs.data_sector_lba(self))
	}
}

impl BIOSParameterBlock {
	fn root_dir_sectors(&self) -> u16 {
		((self.root_entry_count * 32) + (self.bytes_per_sector - 1))
			/ self.bytes_per_sector
	}

	fn root_dir_sector(&self) -> u32 {
		self.reserved_sector_count as u32
			+ (self.num_fats as u32 * self.fat_sz_16 as u32)
	}
}

pub struct Directory {
	name: String,
	entries: Vec<DirectoryEntry>,
}

// None of this is meant to stay.
impl Directory {
	pub fn root(&self) -> &DirectoryEntry {
		self.entries.get(0)
	}

	pub fn entries(&self) -> &[DirectoryEntry] {
		&self.entries[1..]
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	fn new(name: String, device: u8, lba_sector: u32) -> Self {
		read_sector(device, lba_sector);

		// "16 directories ought to be enough for anybody."
		let mut listing = Directory {
			name,
			entries: Vec::new(16),
		};

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
}

pub struct FATFileSystem {
	bpb: BIOSParameterBlock,
	lba_start: u32,
	device: u8,
}

impl FATFileSystem {
	const MBR_PART_1: u32 = 0x1BE;

	pub fn open(device: u8) -> Self {
		read_sector(device, 0);
		let partition_1 = unsafe { read_offset::<Partition>(Self::MBR_PART_1) };

		read_sector(device, partition_1.lba_start);
		Self {
			bpb: unsafe { read_offset::<BIOSParameterBlock>(0x0B) },
			lba_start: partition_1.lba_start,
			device,
		}
	}

	pub fn read_dir(&self, entry: &DirectoryEntry) -> Directory {
		entry.as_dir(self)
	}

	pub fn read_root(&self) -> Directory {
		let mut name = String::new(3);
		name.write_str("A:\\").unwrap();
		Directory::new(name, self.device, self.root_dir_sector_lba())
	}

	pub fn read_file(&self, entry: &DirectoryEntry) -> Vec<u8> {
		assert!(!entry.is_dir());
		assert!(entry.size < 512);

		read_sector(self.device, self.data_sector_lba(entry));

		unsafe { read_offset_to_vec(0, entry.size as usize) }
	}

	fn root_dir_sector_lba(&self) -> u32 {
		self.bpb.root_dir_sector() + self.lba_start
	}

	fn data_sector_lba(&self, entry: &DirectoryEntry) -> u32 {
		((entry.first_cluster_lo as u32 - 2)
			* self.bpb.sectors_per_cluster as u32)
			+ self.root_dir_sector_lba()
			+ self.bpb.root_dir_sectors() as u32
	}
}

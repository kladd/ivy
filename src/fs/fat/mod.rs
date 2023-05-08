pub mod inode;

use alloc::{string::String, vec::Vec};
use core::fmt::{Debug, Write};

use log::info;

use crate::{
	arch::x86::ide::{
		read_offset, read_offset_to_vec, read_sector, write_sector,
	},
	fs::{fat::inode::FATInode, inode::Inode},
	time::DateTime,
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

#[derive(Copy, Clone, Debug, Default)]
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FATFileSystem {
	pub pb: ParameterBlock,
	pub lba_start: u32,
	pub device: u8,
}

impl DirectoryEntry {
	const ATTR_READ_ONLY: u8 = 0x01;
	const ATTR_HIDDEN: u8 = 0x02;
	const ATTR_SYSTEM: u8 = 0x04;
	const ATTR_VOLUME_ID: u8 = 0x08;
	const ATTR_DIRECTORY: u8 = 0x10;
	const ATTR_ARCHIVE: u8 = 0x20;
	const ATTR_LFN: u8 = 0x0F;

	const SIZE: u32 = 32;

	pub fn name(&self) -> String {
		let name = unsafe { core::str::from_utf8_unchecked(&self.name) }
			.split_ascii_whitespace()
			.next()
			.expect("invalid file name");
		let ext = unsafe { core::str::from_utf8_unchecked(&self.ext) }
			.split_ascii_whitespace()
			.next();

		let mut filename = String::with_capacity(12);

		write!(filename, "{}", name).expect("write filename");
		if let Some(ext) = ext {
			write!(filename, ".{}", ext).expect("write extension");
		}

		filename
	}

	pub fn size(&self) -> u32 {
		self.size
	}

	pub fn is_dir(&self) -> bool {
		self.attributes & DirectoryEntry::ATTR_DIRECTORY != 0
	}

	pub fn is_normal(&self) -> bool {
		self.attributes & Self::ATTR_VOLUME_ID & Self::ATTR_LFN == 0
	}

	pub fn present(&self) -> Option<Self> {
		if self.is_empty() {
			None
		} else {
			Some(*self)
		}
	}

	pub fn new(name: &str) -> Self {
		let mut entry = DirectoryEntry::default();
		let (mut name, mut ext) = {
			let mut parts = name.split(".");
			(
				parts.next().unwrap().as_bytes().iter(),
				parts.next().unwrap_or_default().as_bytes().iter(),
			)
		};

		entry
			.name
			.fill_with(|| name.next().map(|b| b.clone()).unwrap_or(0x20u8));
		entry.attributes = DirectoryEntry::ATTR_ARCHIVE;
		entry
			.ext
			.fill_with(|| ext.next().map(|b| b.clone()).unwrap_or(0x20u8));

		let now = DateTime::now();
		let date = as_date(&now);
		let time = as_time(&now);

		entry.m_date = date;
		entry.m_time = time;
		entry.a_date = date;
		entry.c_date = date;
		entry.c_time = time;

		entry
	}

	fn is_empty(&self) -> bool {
		self.name[0] == 0
	}
}

fn as_date(dt: &DateTime) -> u16 {
	// [15-9](years since 1980), [8-5](month), [4-0](day of month)
	(dt.year() - 1980) << 9 | (dt.month() as u16) << 5 | dt.day() as u16
}

fn as_time(dt: &DateTime) -> u16 {
	// [15-11](hours), [10-5](minutes), [4-0](2-seconds??)
	(dt.hour() as u16) << 11
		| (dt.minute() as u16) << 5
		| (dt.second() / 2) as u16
}

impl ParameterBlock {
	fn root_dir_sectors(&self) -> u16 {
		((self.root_entry_count * 32) + (self.bytes_per_sector - 1))
			/ self.bytes_per_sector
	}

	fn root_dir_sector(&self) -> u32 {
		self.reserved_sector_count + (self.num_fats * self.fat_sz_16)
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ParameterBlock {
	root_entry_count: u16,
	bytes_per_sector: u16,
	reserved_sector_count: u32,
	num_fats: u32,
	fat_sz_16: u32,
	sectors_per_cluster: u32,
}

impl From<BIOSParameterBlock> for ParameterBlock {
	fn from(bpb: BIOSParameterBlock) -> Self {
		Self {
			root_entry_count: bpb.root_entry_count,
			bytes_per_sector: bpb.bytes_per_sector,
			reserved_sector_count: bpb.reserved_sector_count as u32,
			num_fats: bpb.num_fats as u32,
			fat_sz_16: bpb.fat_sz_16 as u32,
			sectors_per_cluster: bpb.sectors_per_cluster as u32,
		}
	}
}

impl FATFileSystem {
	const MBR_PART_1: u32 = 0x1BE;
	const END_OF_CHAIN: u16 = 0xFFFF;

	pub fn new(device: u8) -> Self {
		read_sector(device, 0);
		let partition_1 =
			unsafe { read_offset::<Partition>(Self::MBR_PART_1 as usize) };

		read_sector(device, partition_1.lba_start);
		let bpb = unsafe { read_offset::<BIOSParameterBlock>(0x0B) };
		Self {
			pb: ParameterBlock::from(bpb),
			lba_start: partition_1.lba_start,
			device,
		}
	}

	pub fn root(&self) -> Inode {
		let mut entry = DirectoryEntry::default();
		entry.attributes = DirectoryEntry::ATTR_DIRECTORY;
		Inode::FAT(FATInode {
			entry,
			cluster_offset: 0,
			dir_sector: 0,
			fs: self.clone(),
		})
	}

	pub fn read_file(&self, entry: &DirectoryEntry) -> Vec<u8> {
		assert!(!entry.is_dir());
		assert!(entry.size < 512);

		read_sector(self.device, self.data_sector_lba(entry));

		unsafe { read_offset_to_vec(0, entry.size as usize) }
	}

	pub fn write_file(&self, file: &mut DirectoryEntry, data: &[u8]) {
		if file.first_cluster_lo == 0 {
			// unsafe cast, TODO: use hi/lo and shift.
			file.first_cluster_lo = self.alloc_cluster() as u16;
		}
		file.size = data.len() as u32;

		let data_sector = self.data_sector_lba(file);
		write_sector(
			self.device,
			data_sector,
			data as *const [u8] as *const u8 as u32,
		);
	}

	fn root_dir_sector_lba(&self) -> u32 {
		self.pb.root_dir_sector() + self.lba_start
	}

	fn data_sector_lba(&self, entry: &DirectoryEntry) -> u32 {
		let root_dir_lba = self.root_dir_sector_lba();
		match entry.first_cluster_lo.checked_sub(2) {
			Some(first_cluster) => {
				(first_cluster as u32 * self.pb.sectors_per_cluster as u32)
					+ root_dir_lba + self.pb.root_dir_sectors() as u32
			}
			None => root_dir_lba as u32,
		}
	}

	fn read_fat(&self) -> [u16; 32] {
		read_sector(self.device, self.fat_lba(0));
		unsafe { read_offset::<_>(0) }
	}

	fn write_fat(&self, fat: &[u16; 32]) {
		write_sector(self.device, self.fat_lba(0), fat as *const _ as u32);
	}

	fn fat_lba(&self, n: u32) -> u32 {
		self.pb.reserved_sector_count as u32
			+ self.lba_start
			+ (self.pb.fat_sz_16 as u32 * n)
	}

	fn alloc_cluster(&self) -> usize {
		let mut cluster = 0;
		let mut fat = self.read_fat();
		for (n, value) in fat.iter_mut().enumerate() {
			if *value == 0 {
				*value = Self::END_OF_CHAIN;
				cluster = n;
				break;
			}
		}
		assert_ne!(cluster, 0);
		self.write_fat(&fat);
		info!("Allocate cluster {cluster:?}");
		cluster
	}

	// pub fn create(&self, cwd: &FATInode, name: &str) -> FATInode {
	// 	trace!("CREATE({name})");
	// 	let de = DirectoryEntry::new(name);
	// 	let de_cluster = self.data_sector_lba(&cwd.entry);
	//
	// 	let mut dir = self.open(*cwd);
	// 	let mut buf = [0u8; 512];
	// 	assert_eq!(dir.read(&mut buf), 512);
	//
	// 	// Write new directory entry into first available entry in `cwd`.
	// 	let mut entries =
	// 		unsafe { mem::transmute::<[u8; 512], [DirectoryEntry; 16]>(buf) };
	// 	let (offset, entry) = entries
	// 		.iter_mut()
	// 		.enumerate()
	// 		.find(|(_, entry)| entry.is_empty())
	// 		.expect("no available directory entry");
	// 	*entry = de;
	//
	// 	// Write `cwd`'s entries to disk.
	// 	dir.seek(0);
	// 	dir.write(unsafe {
	// 		from_raw_parts(&entries as *const _ as *const u8, 512)
	// 	});
	//
	// 	FATInode {
	// 		dir_sector: de_cluster,
	// 		cluster_offset: offset,
	// 		entry: de,
	// 		fs: self,
	// 	}
	// }
	//
	// pub fn find(&self, base: &FATInode, path: &str) -> Option<FATInode> {
	// 	if path.starts_with("/") {
	// 		self.find(&self.root(), &path[1..])
	// 	} else {
	// 		let segments = path.split("/");
	// 		let mut node = base.clone();
	//
	// 		for segment in segments {
	// 			if segment.is_empty() {
	// 				continue;
	// 			}
	// 			node = self.find_child(node, segment)?;
	// 		}
	//
	// 		Some(node)
	// 	}
	// }
	//
	// fn find_child(&self, base: FATInode, name: &str) -> Option<FATInode> {
	// 	assert!(base.entry.is_dir());
	// 	let mut dir = self.open(base);
	//
	// 	let mut dir_buf = [0u8; DirectoryEntry::SIZE as usize * 16];
	// 	assert_eq!(dir.read(&mut dir_buf), 512);
	//
	// 	let entries = unsafe {
	// 		mem::transmute::<[u8; 512], [DirectoryEntry; 16]>(dir_buf)
	// 	};
	//
	// 	entries
	// 		.into_iter()
	// 		.enumerate()
	// 		.find(|(_, entry)| entry.name() == name)
	// 		.map(|(offset, entry)| FATInode {
	// 			entry,
	// 			cluster_offset: offset,
	// 			dir_sector: self.data_sector_lba(&base.entry),
	// 		})
	// }
	//
	// fn update(&self, node: &FATInode) {
	// 	read_sector(self.device, node.dir_sector);
	//
	// 	let mut entries = unsafe { read_offset::<[DirectoryEntry; 16]>(0) };
	// 	entries[node.cluster_offset] = node.entry;
	//
	// 	write_sector(
	// 		self.device,
	// 		node.dir_sector,
	// 		&entries as *const _ as *const u8 as u32,
	// 	);
	// }
}

// impl<'a> File<'a> {
// 	pub fn tell(&self) -> usize {
// 		self.offset
// 	}
//
// 	pub fn seek(&mut self, pos: usize) {
// 		self.offset = pos;
// 	}
//
// 	pub fn read(&mut self, buf: &mut [u8]) -> usize {
// 		assert!(self.offset + buf.len() <= 512);
//
// 		if self.buffer.is_none() {
// 			read_sector(
// 				self.fs.device,
// 				self.fs.data_sector_lba(&self.node.entry),
// 			);
// 			read(0, self.size(), self.buffer.insert([0u8; 512]));
// 		}
//
// 		let count = if self.node.entry.is_dir() {
// 			min(512 - self.offset, buf.len())
// 		} else {
// 			min(self.node.entry.size as usize - self.offset, buf.len())
// 		};
//
// 		buf[..count].clone_from_slice(
// 			&self.buffer.unwrap()[self.offset..self.offset + count],
// 		);
//
// 		self.offset += count;
//
// 		count
// 	}
//
// 	pub fn write(&mut self, buf: &[u8]) -> usize {
// 		// TODO: Write across sector boundaries.
// 		assert!(self.offset + buf.len() <= 512);
//
// 		if self.node.entry.first_cluster_lo == 0 {
// 			self.node.entry.first_cluster_lo = self.fs.alloc_cluster() as u16;
// 		}
//
// 		if !self.node.entry.is_dir() {
// 			self.node.entry.size += buf.len() as u32;
// 		}
//
// 		read_sector(self.fs.device, self.fs.data_sector_lba(&self.node.entry));
//
// 		let size = if self.node.entry.is_dir() {
// 			512
// 		} else {
// 			self.node.entry.size() as usize + buf.len()
// 		};
//
// 		let mut disk_buf = vec![0u8; size];
// 		read(0, size, &mut disk_buf);
//
// 		for i in 0..buf.len() {
// 			disk_buf[self.offset + i] = buf[i];
// 		}
//
// 		write_sector(
// 			self.fs.device,
// 			self.fs.data_sector_lba(&self.node.entry),
// 			&buf[..] as *const [u8] as *const u8 as u32,
// 		);
// 		self.fs.update(&self.node);
//
// 		buf.len()
// 	}
//
// 	pub fn size(&self) -> usize {
// 		if self.node.entry.is_dir() {
// 			512
// 		} else {
// 			self.node.entry.size as usize
// 		}
// 	}
//
// 	pub fn name(&self) -> String {
// 		self.node.entry.name()
// 	}
//
// 	pub fn entry(&self) -> FATInode {
// 		self.node
// 	}
// }
//
// impl<'a> Write for File<'a> {
// 	fn write_str(&mut self, s: &str) -> core::fmt::Result {
// 		self.write(s.as_bytes());
// 		Ok(())
// 	}
// }

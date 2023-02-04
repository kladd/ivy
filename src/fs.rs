use core::fmt::{Debug, Write};

use crate::arch::x86::ide::{read_offset, read_sector};

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

pub fn read_block_1() {
	// Read MBR.
	read_sector(0);

	// Read first partition table entry.
	let partition_0 = unsafe { read_offset::<Partition>(0x1BE) };
	kdbg!(&partition_0);

	// Read VBR.
	read_sector(partition_0.lba_start);
}

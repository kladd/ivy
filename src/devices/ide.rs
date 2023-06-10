use alloc::vec::Vec;

use crate::arch::amd64::{
	idt::{register_handler, Interrupt},
	inb, insl, outb, outsl,
};

pub const SECTOR_SIZE: usize = 512;

const IDE_BSY: u8 = 0x80;
const IDE_DRDY: u8 = 0x40;
const IDE_ERR: u8 = 0x01;
const IDE_DF: u8 = 0x20;
const IDE_CMD_READ: u8 = 0x20;
const IDE_CMD_WRITE: u8 = 0x30;
const IDE_CMD_FLUSH: u8 = 0xE7;

const LBA_MODE: u8 = 0xE0;

const HDA: u8 = 0;

static mut BUFFER: [u8; SECTOR_SIZE] = [0xFF; SECTOR_SIZE];

fn lba(index: u8) -> u8 {
	index << 4
}

pub fn ide_wait() {
	while (inb(0x1F7) & (IDE_BSY | IDE_DRDY)) != IDE_DRDY { /* SPIN WAIT */ }
	if ((inb(0x1F7) & (IDE_BSY | IDE_DRDY)) & (IDE_DF | IDE_ERR)) != 0 {
		panic!("IDE_ERR");
	}
}

pub fn init_ide() {
	register_handler(46, ide_isr);

	outb(0x1F6, LBA_MODE | lba(1));
	outb(0x1F6, LBA_MODE | lba(HDA));
}

pub fn read_sector(device: u8, sector: u32) {
	ide_wait();

	outb(0x3F6, 0);
	outb(0x1F6, LBA_MODE | lba(device) | (sector >> 24) as u8 & 0x0F); // 24..28 bits of LBA.

	outb(0x1F2, 0x01); // Number of sectors to read.

	outb(0x1F3, sector as u8); // 0..8 bits of LBA.
	outb(0x1F4, (sector >> 8) as u8); // 8..16 bits of LBA.
	outb(0x1F5, (sector >> 16) as u8); // 16..24 bits of LBA.

	outb(0x1F7, IDE_CMD_READ); // Send read command.

	ide_wait();
}

pub fn write_sector(device: u8, sector: usize, src: usize) {
	ide_wait();

	outb(0x3F6, 0);
	outb(0x1F6, LBA_MODE | lba(device) | (sector >> 24) as u8 & 0x0F); // 24..28 bits of LBA.

	outb(0x1F2, 0x01); // Number of sectors to read.

	outb(0x1F3, sector as u8); // 0..8 bits of LBA.
	outb(0x1F4, (sector >> 8) as u8); // 8..16 bits of LBA.
	outb(0x1F5, (sector >> 16) as u8); // 16..24 bits of LBA.

	outb(0x1F7, IDE_CMD_WRITE); // Send write command.
	outsl(SECTOR_SIZE / 4, src, 0x1F0);

	ide_wait();
}

pub fn read(offset: usize, len: usize, buf: &mut [u8]) {
	for i in 0..len {
		buf[i] = unsafe { BUFFER[i + offset] }
	}
}

pub unsafe fn read_offset<T: Copy>(offset: usize) -> T {
	let offset_ptr = &BUFFER as *const [u8; 512] as usize + offset;
	*(offset_ptr as *const T)
}

pub unsafe fn read_offset_to_vec(offset: usize, count: usize) -> Vec<u8> {
	let src = &BUFFER as *const u8;
	unsafe {
		Vec::from_raw_parts(
			src.offset(offset as isize) as *mut u8,
			count,
			count,
		)
	}
}

extern "x86-interrupt" fn ide_isr(int: Interrupt) {
	let buf = unsafe { &BUFFER as *const [u8; SECTOR_SIZE] as usize };
	insl(buf, SECTOR_SIZE / 4, 0x1F0);
	Interrupt::eoi(46);
}

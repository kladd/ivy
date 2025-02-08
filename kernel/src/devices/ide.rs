use alloc::{alloc::alloc, boxed::Box, vec, vec::Vec};
use core::{alloc::Layout, cmp::min, ptr};

use log::trace;

use crate::{
	arch::amd64::{
		idt::{register_handler, Interrupt},
		inb, insl, outb, outsl,
	},
	sync::SpinLock,
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

static BUFFER: SpinLock<[u8; SECTOR_SIZE]> = SpinLock::new([0xFF; SECTOR_SIZE]);

fn lba(index: u8) -> u8 {
	index << 4
}

pub fn ide_wait() {
	while (inb(0x1F7) & (IDE_BSY | IDE_DRDY)) != IDE_DRDY { /* SPIN WAIT */ }
	if ((inb(0x1F7) & (IDE_BSY | IDE_DRDY)) & (IDE_DF | IDE_ERR)) != 0 {
		panic!("IDE_ERR");
	}
}

pub fn init() {
	trace!("ide::init()");
	register_handler(46, ide_isr);

	outb(0x1F6, LBA_MODE | lba(1));
	outb(0x1F6, LBA_MODE | lba(HDA));
}

fn read_sector(device: u8, sector: u32, num_sectors: u8) {
	trace!("ide::read_sector({device}, {sector}, {num_sectors})");
	ide_wait();

	outb(0x3F6, 0);
	outb(0x1F6, LBA_MODE | lba(device) | (sector >> 24) as u8 & 0x0F); // 24..28 bits of LBA.

	outb(0x1F2, num_sectors); // Number of sectors to read.

	outb(0x1F3, sector as u8); // 0..8 bits of LBA.
	outb(0x1F4, (sector >> 8) as u8); // 8..16 bits of LBA.
	outb(0x1F5, (sector >> 16) as u8); // 16..24 bits of LBA.

	outb(0x1F7, IDE_CMD_READ); // Send read command.

	ide_wait();
}

fn write_sector(device: u8, sector: usize, src: usize) {
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

pub fn read(
	device: u8,
	start_sector: u32,
	read_offset: usize,
	dst: *mut u8,
	len: usize,
) {
	let num_sectors = len.div_ceil(SECTOR_SIZE);
	let start_sector = start_sector + (read_offset as u32 / SECTOR_SIZE as u32);
	let read_offset = read_offset % SECTOR_SIZE;

	let mut bytes_read = 0;
	let read_len = min(SECTOR_SIZE - read_offset, len - bytes_read);

	read_sector(device, start_sector, 1);
	read_bytes(read_offset, bytes_read, read_len, dst);

	bytes_read += read_len;

	for i in 1..num_sectors {
		let read_len = min(SECTOR_SIZE, len - bytes_read);

		read_sector(device, start_sector + i as u32, 1);
		read_bytes(0, bytes_read, read_len, dst);

		bytes_read += read_len;
	}
}

pub fn read_type<T>(device: u8, start_sector: u32) -> Box<T> {
	let layout = Layout::new::<T>();
	let buf = unsafe { alloc(layout) };

	let len = layout.size();
	let num_sectors = len.div_ceil(SECTOR_SIZE);

	let mut bytes_read = 0;
	for i in 0..num_sectors {
		let read_len = min(SECTOR_SIZE, len - bytes_read);

		read_sector(device, start_sector + i as u32, 1);
		read_bytes(0, bytes_read, read_len, buf);

		bytes_read += read_len;
	}

	unsafe { Box::from_raw(buf as *mut T) }
}

pub fn read_sector_bytes(device: u8, start_sector: u32) -> Vec<u8> {
	let mut result = vec![0; 512];
	read_sector(device, start_sector, 1);
	read_bytes(0, 0, result.len(), result.as_mut_ptr());
	result
}

pub fn read_offset<T>(
	device: u8,
	start_sector: u32,
	read_offset: usize,
) -> Box<T> {
	assert!(read_offset < SECTOR_SIZE);
	assert!(size_of::<T>() < SECTOR_SIZE);

	let buf = unsafe { alloc(Layout::new::<T>()) };

	read_sector(device, start_sector, 1);
	read_bytes(read_offset, 0, size_of::<T>(), buf);

	unsafe { Box::from_raw(buf as *mut T) }
}

fn read_bytes(
	read_offset: usize,
	write_offset: usize,
	len: usize,
	buf: *mut u8,
) {
	let mut guard = BUFFER.lock();
	unsafe {
		ptr::copy_nonoverlapping(
			guard.as_mut_ptr().offset(read_offset as isize),
			buf.offset(write_offset as isize),
			len,
		)
	};
}

fn read_buffer(offset: usize, len: usize, buf: &mut [u8]) {
	read_bytes(offset, 0, len, buf as *mut _ as *mut u8);
}

extern "x86-interrupt" fn ide_isr(int: Interrupt) {
	trace!("IDE INTERRUPT: {int:#?}");
	let guard = BUFFER.lock();
	let buf = guard.as_ref() as *const _ as *mut u8 as usize;
	insl(buf, SECTOR_SIZE / 4, 0x1F0);
	Interrupt::eoi(46);
}

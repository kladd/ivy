use core::fmt::Write;

use crate::{
	arch::x86::interrupt_descriptor_table::{
		register_handler, InterruptRequest,
	},
	isr,
	x86::common::{inb, insl, outb},
};

pub const SECTOR_SIZE: usize = 512;

const IDE_BSY: u8 = 0x80;
const IDE_DRDY: u8 = 0x40;
const IDE_ERR: u8 = 0x01;
const IDE_DF: u8 = 0x20;
const IDE_CMD_READ: u8 = 0x20;

const LBA_MODE: u8 = 0xE0;

const HDA: u8 = 0;

static mut BUFFER: [u8; SECTOR_SIZE] = [0xFF; SECTOR_SIZE];

fn lba(index: u8) -> u8 {
	index << 4
}

pub fn ide_wait() {
	let status = inb(0x1F7) & (IDE_BSY | IDE_DRDY);
	kdbg!(status);
	while status != IDE_DRDY {
		// wait!
	}
	if (status & (IDE_DF | IDE_ERR)) != 0 {
		panic!("IDE_ERR");
	}
	kprintf!("IDE: READY");
}

pub fn ide_init() {
	register_handler(isr!(46, ide_isr));

	outb(0x1F6, LBA_MODE | lba(1));
	if inb(0x1F7) != 0 {
		kprintf!("IDE: DISK1 PRESENT");
	} else {
		kprintf!("IDE: DISK1 NOT PRESENT");
	}
	outb(0x1F6, LBA_MODE | lba(HDA));
}

pub fn read_sector(sector: u32) {
	ide_wait();

	outb(0x3F6, 0);
	outb(
		0x1F6,
		kdbg!(LBA_MODE | lba(HDA) | (sector >> 24) as u8 & 0x0F),
	); // 24..28 bits of LBA.

	outb(0x1F2, 0x01); // Number of sectors to read.

	outb(0x1F3, kdbg!(sector as u8)); // 0..8 bits of LBA.
	outb(0x1F4, kdbg!((sector >> 8) as u8)); // 8..16 bits of LBA.
	outb(0x1F5, kdbg!((sector >> 16) as u8)); // 16..24 bits of LBA.

	outb(0x1F7, IDE_CMD_READ); // Send read command.

	// TODO: Sync.
}

pub unsafe fn read_offset<T: Copy>(offset: u32) -> T {
	let offset_ptr = &BUFFER as *const [u8; 512] as u32 + offset;

	*(offset_ptr as *const T)
}

pub fn ide_isr(int: &InterruptRequest) {
	kprintf!("ide isr");
	kdbg!(int);

	let buf = unsafe { &BUFFER as *const [u8; SECTOR_SIZE] as u32 };
	ide_wait();
	insl(0x1F0, buf, (SECTOR_SIZE / 4) as u32);

	int.eoi();
}

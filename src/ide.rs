use core::fmt::Write;

use crate::{
	arch::x86::interrupt_descriptor_table::{
		register_handler, InterruptStackFrame,
	},
	isr,
	x86::common::{inb, insl, outb},
};

const SECTOR_SIZE: usize = 512;

const IDE_BSY: u8 = 0x80;
const IDE_DRDY: u8 = 0x40;
const IDE_ERR: u8 = 0x01;
const IDE_DF: u8 = 0x20;
const IDE_CMD_READ: u8 = 0x20;

static mut BUFFER: [u8; SECTOR_SIZE] = [0xFF; SECTOR_SIZE];

fn lba(index: u8) -> u8 {
	index << 4
}

pub fn ide_wait() {
	let status = inb(0x1F7) & (IDE_BSY | IDE_DRDY);
	while status != IDE_DRDY {
		// wait!
	}
	if (status & (IDE_DF | IDE_ERR)) != 0 {
		panic!("IDE_ERR");
	}
	kprintf!("IDE: READY");
}

pub fn ide_init() {
	register_handler(isr!(46, ide_irq));

	outb(0x1F6, 0xE0 | lba(1));
	if inb(0x1F7) != 0 {
		kprintf!("IDE: DISK1 PRESENT");
	} else {
		kprintf!("IDE: DISK1 NOT PRESENT");
	}
	outb(0x1F6, 0xE0 | lba(0));
}

pub fn read_block_1() {
	ide_wait();

	let sector: u32 = 6;

	outb(0x3F6, 0); // generate interrupt?
	outb(0x1F2, 1); // number of sectors.
	outb(0x1F3, (sector & 0xFF) as u8); // ?
	outb(0x1F4, ((sector >> 8) & 0xFF) as u8);
	outb(0x1F5, ((sector >> 16) & 0xFF) as u8);
	outb(0x1F6, 0xE0 | lba(1) | ((sector >> 24) & 0x0F) as u8);
	outb(0x1F7, IDE_CMD_READ);

	unsafe {
		kdbg!(BUFFER);
	}
}

pub fn ide_irq(stack_frame: &InterruptStackFrame) {
	kprintf!("ide isr");
	kdbg!(stack_frame);

	let buf = unsafe { &BUFFER as *const [u8; SECTOR_SIZE] as u32 };
	ide_wait();
	insl(0x1F0, buf, (SECTOR_SIZE / 4) as u32);
}

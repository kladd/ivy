use crate::arch::amd64::{inb, outb};

const PIC1: u16 = 0x20;
const PIC1_CMD: u16 = PIC1;
const PIC1_DAT: u16 = PIC1 + 1;

const PIC2: u16 = 0xA0;
const PIC2_CMD: u16 = PIC2;
const PIC2_DAT: u16 = PIC2 + 1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;

const ICW4_8086: u8 = 0x01;

pub fn init_pic() {
	// Save masks.
	let pic1_mask = inb(PIC1_DAT);
	let pic2_mask = inb(PIC2_DAT);

	// Send init.
	outb(PIC1_CMD, ICW1_INIT | ICW1_ICW4);
	outb(PIC2_CMD, ICW1_INIT | ICW1_ICW4);

	// ICW 2
	outb(PIC1_DAT, 0x20);
	outb(PIC2_DAT, 0x28);

	// ICW 3
	outb(PIC1_DAT, 4);
	outb(PIC2_DAT, 2);

	// ICW 4
	outb(PIC1_DAT, ICW4_8086);
	outb(PIC2_DAT, ICW4_8086);

	// Restore masks.
	outb(PIC1_DAT, pic1_mask);
	outb(PIC2_DAT, pic2_mask);
}

use core::arch::asm;

pub mod clock;
mod descriptor_table;
pub mod global_descriptor_table;
pub mod ide;
pub mod interrupt_controller;
pub mod interrupt_descriptor_table;

pub fn enable_interrupts() {
	unsafe { asm!("sti") }
}

pub fn disable_interrupts() {
	unsafe { asm!("cli") }
}

pub fn halt() {
	unsafe { asm!("hlt") }
}

/// x86 OUT instruction for byte operands.
pub fn outb(port: u16, b: u8) {
	unsafe {
		// Output byte in al to I/O port address in dx.
		asm!("out dx, al", in("dx") port, in("al") b);
	}
}

pub fn inb(port: u16) -> u8 {
	let mut b: u8;
	unsafe {
		asm!("in al, dx", in("dx") port, out("al") b);
	}
	b
}

pub fn insl(port: u16, out: u32, count: u32) {
	unsafe { insl_asm(port, out, count) }
}

pub fn outsl(port: u16, src: u32, count: u32) {
	unsafe { outsl_asm(port, src, count) }
}

extern "C" {
	pub fn insl_asm(port: u16, out: u32, count: u32);
	pub fn outsl_asm(port: u16, src: u32, count: u32);
}

use core::arch::asm;

pub mod clock;
pub mod gdt;
pub mod idt;
pub mod pic;
pub mod vmem;

pub fn sti() {
	unsafe { asm!("sti") }
}

pub fn cli() {
	unsafe { asm!("cli") }
}

pub fn hlt() {
	unsafe { asm!("hlt") }
}

pub fn outb(port: u16, b: u8) {
	unsafe {
		// Output byte in al to I/O port address in dx.
		asm!("out dx, al", in("dx") port, in("al") b);
	}
}

pub fn outl(port: u16, l: u32) {
	unsafe { asm!("out dx, eax", in("dx") port, in("eax") l) };
}

pub fn inl(port: u16) -> u32 {
	let mut l: u32;
	unsafe { asm!("in eax, dx", in("dx") port, out("eax") l) };
	l
}

pub fn inw(port: u16) -> u16 {
	let mut w: u16;
	unsafe { asm!("in eax, dx", in("dx") port, out("ax") w) };
	w
}

pub fn outsl(count: usize, src: usize, port: u16) {
	unsafe { outsl_asm(count, src, port) }
}

pub fn insl(dst: usize, count: usize, port: u16) {
	unsafe { insl_asm(dst, count, port) }
}

pub fn inb(port: u16) -> u8 {
	let mut b: u8;
	unsafe {
		asm!("in al, dx", in("dx") port, out("al") b);
	}
	b
}

extern "C" {
	fn outsl_asm(count: usize, src: usize, port: u16);
	fn insl_asm(dst: usize, len: usize, port: u16);
}

use core::arch::asm;

pub mod clock;
pub mod interrupts;
pub mod pic;

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

pub fn inb(port: u16) -> u8 {
	let mut b: u8;
	unsafe {
		asm!("in al, dx", in("dx") port, out("al") b);
	}
	b
}

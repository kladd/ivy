use core::arch::asm;

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

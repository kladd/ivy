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

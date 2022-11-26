use core::arch::asm;

/// x86 HLT instruction.
pub fn halt() -> ! {
	unsafe {
		asm!("hlt");
	}
	unreachable!();
}

/// x86 OUT instruction for byte operands.
pub fn outb(b: u8, port: u16) {
	unsafe {
		// Output byte in al to I/O port address in dx.
		asm!("out dx, al", in("al") b, in("dx") port);
	}
}

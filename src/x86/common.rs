use core::arch::asm;

/// x86 HLT instruction.
pub fn halt() -> ! {
	unsafe {
		asm!("cli", "hlt");
	}
	unreachable!();
}

/// x86 OUT instruction for byte operands.
pub fn outb(port: u16, b: u8) {
	unsafe {
		// Output byte in al to I/O port address in dx.
		asm!("out dx, al", in("dx") port, in("al") b);
	}
}

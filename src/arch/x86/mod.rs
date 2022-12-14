use core::arch::asm;

mod descriptor_table;
pub mod global_descriptor_table;
pub mod interrupt_controller;
pub mod interrupt_descriptor_table;
pub mod timer;

pub fn enable_interrupts() {
	unsafe {
		asm!("sti");
	}
}

pub fn disable_interrupts() {
	unsafe {
		asm!("cli");
	}
}

/// x86 HLT instruction.
pub fn halt() -> ! {
	unsafe { asm!("hlt", options(noreturn)) }
}

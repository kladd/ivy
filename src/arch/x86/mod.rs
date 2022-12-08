use core::arch::asm;

mod descriptor_table;
pub mod global_descriptor_table;
pub mod interrupt_descriptor_table;

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

pub fn int3() {
	unsafe {
		asm!("int 3");
	}
}

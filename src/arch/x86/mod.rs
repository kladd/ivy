use core::{
	arch::{asm, global_asm},
	include_str,
};

mod descriptor_table;
pub mod global_descriptor_table;
pub mod interrupt_descriptor_table;

#[link_section = ".multiboot"]
static _MULTIBOOT_MAGIC: i32 = 0x1BADB002;
#[link_section = ".multiboot"]
// ALIGN | MEM_INFO
static _MULTIBOOT_FLAGS: i32 = 3;
#[link_section = ".multiboot"]
static _MULTIBOOT_CHK: i32 = -(_MULTIBOOT_MAGIC + _MULTIBOOT_FLAGS);

global_asm!(include_str!("boot.asm"));

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

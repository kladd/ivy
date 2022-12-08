#![no_std]
#![no_main]

#[macro_use]
mod debug;
mod arch;
mod serial;
mod vga;
mod x86;

use core::{fmt::Write, panic::PanicInfo};

use crate::{
	arch::x86::{
		disable_interrupts, enable_interrupts,
		global_descriptor_table::init_gdt, int3,
		interrupt_descriptor_table::init_idt,
	},
	serial::COM1,
	vga::VGA,
	x86::{common::halt, pic},
};

pub const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

#[derive(Debug)]
#[repr(C)]
pub struct MultibootInfo {
	flags: u32,
	mem_lower: u32,
	mem_upper: u32,
	// TODO: The rest.
}

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	kprintf!("kernel {}", _info);
	halt()
}

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_info: &MultibootInfo,
	stack_top: u32,
	stack_bottom: u32,
) -> ! {
	assert_eq!(multiboot_magic, MULTIBOOT_MAGIC);
	kdbg!(multiboot_info);
	kprintf!("{:#08X} {:#08X}", stack_bottom, stack_top);

	disable_interrupts();

	init_gdt();
	init_idt();

	enable_interrupts();

	pic::init();

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	int3();

	kprintf!("If you can read this, {} interrupt handling works", "debug");

	VGA.clear_screen();
	VGA.disable_cursor();
	writeln!(VGA, "Welcome to Ivy OS!").unwrap();

	halt()
}

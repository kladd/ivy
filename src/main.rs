#![feature(abi_x86_interrupt)]
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
		global_descriptor_table::{flush_gdt, init_gdt},
		interrupt_descriptor_table::{flush_idt, init_idt},
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
) -> ! {
	assert_eq!(multiboot_magic, MULTIBOOT_MAGIC);

	disable_interrupts();

	kdbg!(multiboot_info);

	let gdt = init_gdt();
	flush_gdt(&gdt);

	let idt = init_idt();
	flush_idt(&idt);

	enable_interrupts();

	pic::init();

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	VGA.clear_screen();
	VGA.disable_cursor();
	writeln!(VGA, "Welcome to Ivy OS!").unwrap();

	halt()
}

#![no_std]
#![no_main]

#[macro_use]
mod debug;
mod boot;
mod serial;
mod vga;
mod x86;

use core::{arch::global_asm, fmt::Write, include_str, panic::PanicInfo};

use crate::{
	boot::{MultibootInfo, MULTIBOOT_MAGIC},
	serial::COM1,
	vga::VGA,
	x86::{
		common::halt,
		gdt::{GlobalDescriptorTableRegister, SegmentDescriptor},
	},
};

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
	kdbg!(multiboot_info);

	let gdt = [
		SegmentDescriptor::null(),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0x9A, 0xCF),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0x92, 0xCF),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0xFA, 0xCF),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0xF2, 0xCF),
	];
	let gdtr = GlobalDescriptorTableRegister::new(gdt);
	unsafe {
		gdtr.flush();
	}

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	VGA.clear_screen();
	VGA.disable_cursor();
	writeln!(VGA, "Welcome to Ivy OS!").unwrap();

	halt()
}

#[link_section = ".multiboot"]
static _MULTIBOOT_MAGIC: i32 = 0x1BADB002;
#[link_section = ".multiboot"]
// ALIGN | MEM_INFO
static _MULTIBOOT_FLAGS: i32 = 3;
#[link_section = ".multiboot"]
static _MULTIBOOT_CHK: i32 = -(_MULTIBOOT_MAGIC + _MULTIBOOT_FLAGS);

global_asm!(include_str!("boot/boot.s"));

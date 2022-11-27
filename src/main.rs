#![no_std]
#![no_main]

#[macro_use]
mod debug;
mod serial;
mod vga;
mod x86;

use core::{fmt::Write, panic::PanicInfo};

use crate::{serial::COM1, vga::VGA, x86::common::halt};

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	halt()
}

#[no_mangle]
pub extern "C" fn start() -> ! {
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

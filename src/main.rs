#![no_std]
#![no_main]

#[macro_use]
mod debug;
mod serial;
mod x86;

use core::{fmt::Write, panic::PanicInfo};

use crate::x86::common::halt;

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	halt()
}

#[no_mangle]
pub extern "C" fn start() -> ! {
	let vga = 0xb8000 as *mut u8;

	unsafe {
		*vga.offset(0isize) = 'O' as u8;
		*vga.offset(2isize) = 'K' as u8;
	}

	serial::init_port(serial::COM1);

	kprintf!("If you can read this, {} logging works", "debug");

	halt()
}

#[link_section = ".multiboot"]
static _MULTIBOOT_MAGIC: i32 = 0x1BADB002;
#[link_section = ".multiboot"]
static _MULTIBOOT_ARCH: i32 = 0;
#[link_section = ".multiboot"]
static _MULTIBOOT_CHK: i32 = -_MULTIBOOT_MAGIC;

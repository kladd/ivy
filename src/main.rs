#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	halt()
}

fn halt() -> ! {
	unsafe {
		asm!("hlt");
	}
	unreachable!();
}

#[no_mangle]
pub extern "C" fn start() -> ! {
	let vga = 0xb8000 as *mut u8;

	unsafe {
		*vga.offset(0isize) = 'O' as u8;
		*vga.offset(2isize) = 'K' as u8;
	}

	halt()
}

#[link_section = ".multiboot"]
static _MULTIBOOT_MAGIC: i32 = 0x1BADB002;
#[link_section = ".multiboot"]
static _MULTIBOOT_ARCH: i32 = 0;
#[link_section = ".multiboot"]
static _MULTIBOOT_CHK: i32 = -_MULTIBOOT_MAGIC;

#![no_std]
#![no_main]

#[macro_use]
mod debug;
mod arch;
mod multiboot;
mod serial;
mod vga;
mod x86;

use core::{fmt::Write, panic::PanicInfo};

use crate::{
	arch::x86::{
		disable_interrupts, enable_interrupts,
		global_descriptor_table::init_gdt, halt,
		interrupt_controller::init_pic, interrupt_descriptor_table::init_idt,
		timer::init_timer,
	},
	multiboot::{MultibootFlags, MultibootInfo},
	serial::COM1,
	vga::VGA,
};

pub const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	disable_interrupts();
	kprintf!("kernel {}", _info);
	halt()
}

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_flags: &MultibootFlags,
) {
	assert_eq!(multiboot_magic, MULTIBOOT_MAGIC);
	let boot_info = MultibootInfo::read(multiboot_flags);
	kdbg!(boot_info);

	init_gdt();
	init_idt();
	init_pic();

	init_timer();

	enable_interrupts();

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	VGA.clear_screen();
	VGA.disable_cursor();
	writeln!(VGA, "Welcome to Ivy OS!").unwrap();
}

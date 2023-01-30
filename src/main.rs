#![no_std]
#![no_main]
#![feature(naked_functions)]

#[macro_use]
mod debug;
mod arch;
mod keyboard;
mod libk;
mod multiboot;
mod serial;
mod vga;
mod x86;

use core::{arch::asm, fmt::Write, panic::PanicInfo};

use crate::{
	arch::x86::{
		disable_interrupts, enable_interrupts,
		global_descriptor_table::init_gdt,
		halt,
		interrupt_controller::init_pic,
		interrupt_descriptor_table::{
			init_idt, register_handler, InterruptStackFrame,
		},
		timer::init_timer,
	},
	keyboard::init_keyboard,
	libk::vec::Vec,
	multiboot::{MultibootFlags, MultibootInfo},
	serial::COM1,
	vga::VGA,
	x86::common::{inb, outb},
};

pub const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

#[repr(align(4096))]
struct PageTable([u32; 1024]);

#[repr(align(4096))]
struct PageDirectory([u32; 512]);

//                                                       // S, R/W, NP
static mut PAGE_DIRECTORY: PageDirectory = PageDirectory([0x00000002; 512]);

static mut PAGE_TABLE_1: PageTable = PageTable([0; 1024]);

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	disable_interrupts();
	kprintf!("kernel {}", _info);
	halt()
}

extern "C" {
	fn enable_paging();
}

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_flags: &MultibootFlags,
) {
	assert_eq!(multiboot_magic, MULTIBOOT_MAGIC);
	let boot_info = MultibootInfo::read(multiboot_flags);
	kdbg!(boot_info);

	unsafe {
		for i in 0..1024 {
			PAGE_TABLE_1.0[i] = (i as u32 * 0x1000) | 3;
		}
		PAGE_DIRECTORY.0[0] = (&PAGE_TABLE_1 as *const PageTable as u32) | 3;

		asm!("mov cr3, {}", in(reg) &PAGE_DIRECTORY as *const PageDirectory as u32);
		enable_paging();
	}

	unsafe {
		VGA.clear_screen();
		writeln!(VGA, "Welcome to Ivy OS!").unwrap();
	}

	init_gdt();
	init_idt();
	init_pic();

	init_timer();
	init_keyboard();

	enable_interrupts();

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	let mut v1 = Vec::new(5);
	v1.push('a');
	v1.push('b');

	let mut v2 = Vec::new(3);
	v2.push('z');
	v2.push('y');

	for i in 0..v1.len() {
		kprintf!("v1[{}] = {}", i, v1.get(i));
	}

	for i in 0..v2.len() {
		kprintf!("v2[{}] = {}", i, v2.get(i));
	}

	dump_register!("cr0");
}

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(dead_code)]
#![allow(unused_macros)]

extern crate alloc;

#[macro_use]
mod debug;
mod arch;
mod ed;
mod fat;
mod fs;
mod keyboard;
mod multiboot;
mod proc;
mod serial;
mod shell;
mod std;
mod time;
mod vga;

use core::{arch::asm, fmt::Write, panic::PanicInfo};

use crate::{
	arch::x86::{
		clock::init_clock, disable_interrupts, enable_interrupts,
		global_descriptor_table::init_gdt, halt, ide::init_ide,
		interrupt_controller::init_pic, interrupt_descriptor_table::init_idt,
	},
	fat::FATFileSystem,
	keyboard::init_keyboard,
	multiboot::{MultibootFlags, MultibootInfo},
	proc::{schedule, Task},
	serial::COM1,
	std::alloc::KernelAlloc,
	vga::VideoMemory,
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
	halt();
	unreachable!();
}

#[global_allocator]
static GLOBAL: KernelAlloc = KernelAlloc;

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

	init_gdt();
	unsafe {
		for i in 0..1024 {
			PAGE_TABLE_1.0[i] = (i as u32 * 0x1000) | 3;
		}
		PAGE_DIRECTORY.0[0] = (&PAGE_TABLE_1 as *const PageTable as u32) | 3;

		asm!("mov cr3, {}", in(reg) &PAGE_DIRECTORY as *const PageDirectory as u32);
		enable_paging();
	}

	let mut vga = VideoMemory::get();

	vga.clear_screen();
	writeln!(vga, "Welcome to Ivy OS!").unwrap();

	init_idt();
	init_pic();

	init_clock();
	init_keyboard();

	enable_interrupts();

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	init_ide();

	// Start the shell.
	let fs = FATFileSystem::new(0);
	let cwd = fs.find(&fs.root(), "HOME/USER").unwrap();
	let sh = Task::new(shell::main, &fs, &cwd);
	schedule(&sh);
}

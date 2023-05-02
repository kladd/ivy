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
mod logger;
mod multiboot;
mod proc;
mod serial;
mod shell;
mod std;
mod time;
mod vga;

use core::{fmt::Write, panic::PanicInfo};

use log::{error, info};

use crate::{
	arch::x86::{
		clock::init_clock, disable_interrupts, enable_interrupts,
		global_descriptor_table::init_gdt, halt, ide::init_ide,
		interrupt_controller::init_pic, interrupt_descriptor_table::init_idt,
		virtual_memory::init_kernel_page_tables,
	},
	fat::FATFileSystem,
	keyboard::init_keyboard,
	logger::KernelLogger,
	multiboot::{MultibootFlags, MultibootInfo},
	proc::{schedule, Task},
	serial::COM1,
	std::alloc::KernelAlloc,
	vga::VideoMemory,
};

pub const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	disable_interrupts();
	error!("kernel {}", _info);
	halt();
	unreachable!();
}

#[global_allocator]
static GLOBAL: KernelAlloc = KernelAlloc;

static LOGGER: KernelLogger = KernelLogger;

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_flags: &MultibootFlags,
) {
	assert_eq!(multiboot_magic, MULTIBOOT_MAGIC);

	COM1.init();
	log::set_logger(&LOGGER).unwrap();
	log::set_max_level(log::STATIC_MAX_LEVEL);

	let _boot_info = MultibootInfo::read(multiboot_flags);

	init_gdt();
	init_kernel_page_tables();

	let mut vga = VideoMemory::get();

	vga.clear_screen();
	writeln!(vga, "Welcome to Ivy OS!").unwrap();

	init_idt();
	init_pic();

	init_clock();
	init_keyboard();

	enable_interrupts();

	info!("If you can read this, info logging works");

	init_ide();

	// Start the shell.
	let fs = FATFileSystem::new(0);
	let cwd = fs.find(&fs.root(), "HOME/USER").unwrap();
	let sh = Task::new(shell::main, &fs, &cwd);
	schedule(&sh);
}

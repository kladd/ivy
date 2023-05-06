#![no_std]
#![no_main]
#![feature(naked_functions)]
#![allow(dead_code)]
#![allow(unused_macros)]

extern crate alloc;

#[macro_use]
mod debug;
mod arch;
mod devices;
// mod ed;
mod fs;
mod logger;
mod multiboot;
// mod proc;
// mod shell;
mod std;
mod time;
mod vga;

use alloc::rc::Rc;
use core::{fmt::Write, panic::PanicInfo};

use log::error;
#[cfg(feature = "headless")]
use log::warn;

#[cfg(feature = "headless")]
use crate::devices::serial::COM1;
use crate::fs::dev::DeviceFileSystem;
#[cfg(not(feature = "headless"))]
use crate::vga::VideoMemory;
use crate::{
	arch::x86::{
		clock::init_clock, disable_interrupts, enable_interrupts,
		global_descriptor_table::init_gdt, halt, ide::init_ide,
		interrupt_controller::init_pic, interrupt_descriptor_table::init_idt,
		virtual_memory::init_kernel_page_tables,
	},
	devices::{keyboard::init_keyboard, serial::init_serial},
	fs::{fat::FATFileSystem, inode::Inode, FileSystem},
	logger::KernelLogger,
	multiboot::{MultibootFlags, MultibootInfo},
	// proc::{schedule, Task},
	std::alloc::KernelAlloc,
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
	let _boot_info = MultibootInfo::read(multiboot_flags);

	init_gdt();
	init_kernel_page_tables();

	log::set_logger(&LOGGER).unwrap();
	log::set_max_level(log::STATIC_MAX_LEVEL);

	init_idt();
	init_pic();

	init_clock();
	init_keyboard();
	init_serial();

	enable_interrupts();

	#[cfg(not(feature = "headless"))]
	{
		let mut vga = VideoMemory::get();
		vga.clear_screen();
		writeln!(vga, "Welcome to Ivy OS!").unwrap();
	}
	#[cfg(feature = "headless")]
	unsafe {
		writeln!(COM1, "\n\nWelcome to Ivy OS!\n").unwrap();
		warn!("To quit type Ctrl-A then `c` then `quit`.\n");
	}

	init_ide();

	// Start the shell.
	let dosfs = Rc::new(FATFileSystem::new(0));
	let devfs = DeviceFileSystem;
	let mut fs = FileSystem::new(Inode::FAT(dosfs.root()));
	fs.mount("DEV", devfs.root_inode());
	kdbg!(fs.open(fs.root(), "HOME/USER/README.MD"));
	kdbg!(fs.open(fs.root(), "DEV/CONSOLE"));
	// let cwd = fs.find(&fs.root(), "HOME/USER").unwrap();
	// let sh = Task::new(shell::main, &fs, &cwd);
	// schedule(&sh);
}

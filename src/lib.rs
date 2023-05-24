#![no_std]
#![feature(abi_x86_interrupt)]
// TODO: Un-suppress these warnings.
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

mod arch;
#[macro_use]
mod debug;
mod devices;
mod kalloc;
mod logger;
mod multiboot;
mod sync;

use alloc::vec;
use core::{panic::PanicInfo, sync::atomic::Ordering};

use log::{debug, error, info};

use crate::{
	arch::amd64::{
		cli,
		clock::init_clock,
		hlt,
		idt::init_idt,
		pic::init_pic,
		sti,
		vmem::{PageTable, BOOT_PD_TABLE},
	},
	devices::{serial::init_serial, video::Video},
	kalloc::init_kalloc,
	logger::KernelLogger,
	multiboot::MultibootInfo,
};

static LOGGER: KernelLogger = KernelLogger;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	error!("kernel {info:#}");
	cli();
	hlt();
	unreachable!();
}

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_info: &MultibootInfo,
) -> ! {
	init_serial();

	log::set_logger(&LOGGER).unwrap();
	log::set_max_level(log::STATIC_MAX_LEVEL);

	debug!("Hello world");

	kdbg!(multiboot_info);

	// let pml4t = unsafe { &*BOOT_PML4_TABLE.load(Ordering::Relaxed) };
	let pdt = unsafe { &mut *BOOT_PD_TABLE.load(Ordering::Relaxed) };
	info!("framebuffer: {:?}", pdt[4]);

	// Framebuffer is 2 < x 4 MB, map the next two pages for it.
	pdt[4] = (multiboot_info.framebuffer_addr | 0x83) as *mut PageTable;
	pdt[5] =
		(multiboot_info.framebuffer_addr + 0x200000 | 0x83) as *mut PageTable;

	// Set the kernel heap to be...0xC00000..2MB?
	pdt[6] = ((6 * 0x200000) | 0x83) as *mut PageTable;
	init_kalloc(6 * 0x200000, 0x200000);

	// Test the heap.
	let vec = vec!["hello", "world"];
	for s in vec {
		debug!("{s}");
	}

	init_idt();
	init_pic();
	init_clock();

	sti();

	let mut screen = Video::new(0x800000);
	screen.test();

	info!("kernel_end: 0x{:016X}", _kernel_end as usize);

	loop {
		hlt()
	}
}

extern "C" {
	fn _kernel_end();
}

#![no_std]
#![feature(abi_x86_interrupt)]

mod arch;
#[macro_use]
mod debug;
mod devices;
mod logger;
mod multiboot;

use core::{arch::asm, ops::Deref, panic::PanicInfo};

use log::{debug, error, info};

use crate::{
	arch::amd64::{
		cli, clock::init_clock, hlt, idt::init_idt, pic::init_pic, sti,
	},
	devices::{serial::init_serial, video::Video},
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
	// let fc_green = Color(0x90, 0xb0, 0x80);
	// init_idt();
	// init_pic();
	// init_clock();
	//
	// sti();
	//
	let mut screen = Video::new(0x800000);
	screen.test();
	// let fb = 0x800000 as *mut u32;
	// for i in 0..(multiboot_info.framebuffer_width
	// 	* multiboot_info.framebuffer_height)
	// {
	// 	unsafe { *fb.offset(i as isize) = fc_green.into() };
	// }
	info!("kernel_end: 0x{:016X}", _kernel_end as usize);
	// // Test interrupts.
	// unsafe { asm!("int 3") };
	//
	loop {
		hlt()
	}
}

extern "C" {
	fn _kernel_end();
}

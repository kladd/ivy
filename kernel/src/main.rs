#![no_std]
#![no_main]

mod dev;
mod sync;
mod logger;

use core::{arch::asm, panic::PanicInfo};
use log::{error, info, warn};
use crate::logger::KernelLogger;

#[unsafe(no_mangle)]
pub static DEBUG_VALUE: u64 = 0xDECAFBAD;

static LOGGER: KernelLogger = KernelLogger;

#[unsafe(no_mangle)]
#[cfg(not(feature = "hardware"))]
pub static ROM_END: u64 = 0x40080000;
#[cfg(feature = "hardware")]
pub static ROM_END: u64 = 0x00080000;

unsafe extern "C" {
	fn _cpu_halt() -> !;
}

#[repr(C)]
pub struct ExceptionContext {
	pub regs: [u64; 31],
	pub elr: u64,
	pub spsr: u64,
	pub esr: u64,
	pub far: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_exception(ctx: &mut ExceptionContext) {
	unsafe { asm!("mov x21, {:x}", in(reg) ctx.elr) };
	panic!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	let line = info.location().map(|l| l.line()).unwrap_or(69);
	let col = info.location().map(|l| l.column()).unwrap_or(69);
	error!("{} {:?}", info.message(), info.location().unwrap());
	unsafe {
		asm!("mov x26, {:x}", in(reg) line);
		asm!("mov x29, {:x}", in(reg) col);
		_cpu_halt();
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start() {
	log::set_logger(&LOGGER).unwrap();
	log::set_max_level(log::STATIC_MAX_LEVEL);

	info!("Hello Ivy");
	warn!("Nothing to do");
	panic!("EXIT");
}

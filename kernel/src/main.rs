#![no_std]
#![no_main]

mod dev;
mod logger;
mod sync;

use core::panic::PanicInfo;

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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	error!("{} {:?}", info.message(), info.location().unwrap());
	unsafe {
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

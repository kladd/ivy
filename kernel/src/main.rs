#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

unsafe extern "C" {
	fn _cpu_halt() -> !;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	unsafe {
		asm!("movz x4, #0xfbad", "movk x4, #0xdeca, lsl #16", out("x4") _);
		_cpu_halt();
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start() -> ! {
	unsafe {
		asm!( "movz x3, #0xfbad", "movk x3, #0xdeca, lsl #16", out("x3") _);
		_cpu_halt();
	}
}

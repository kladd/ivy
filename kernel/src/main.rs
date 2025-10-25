#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[unsafe(no_mangle)]
pub static DEBUG_VALUE: u64 = 0xDECAFBAD;

unsafe extern "C" {
	fn _cpu_halt() -> !;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	unsafe {
		asm!("ldr x4, ={val}", val = const DEBUG_VALUE, out("x4") _);
		_cpu_halt();
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start() -> ! {
	unsafe {
		asm!("ldr x3, ={val}", val = const DEBUG_VALUE, out("x3") _);
		_cpu_halt();
	}
}

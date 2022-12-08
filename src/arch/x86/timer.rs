use core::fmt::Write;

use crate::{
	arch::x86::interrupt_descriptor_table::register_handler, x86::common::outb,
};

static mut CLOCK: u32 = 0;

extern "C" {
	fn interval_timer_handler() -> !;
}

pub fn init_timer() {
	// Set interval interrupt handler.
	register_handler(32, interval_timer_handler as u32);

	// Command byte, set mode.
	outb(0x43, 0x36);

	// Set divisor. This is the slowest possible clock.
	outb(0x40, 0xFF);
	outb(0x40, 0xFF);
}

#[no_mangle]
pub extern "C" fn handle_interval_timer() {
	// Send EOI
	outb(0x20, 0x20);

	unsafe {
		CLOCK += 1;
		kprintf!("TICK: {}", CLOCK);
	}
}

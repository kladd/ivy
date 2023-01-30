use core::fmt::Write;

use crate::{
	arch::x86::interrupt_descriptor_table::register_handler, vga::VGA,
	x86::common::outb,
};

static mut CLOCK: u32 = 0;

const FREQ: u32 = 18;

extern "C" {
	fn interval_timer_handler() -> !;
}

pub fn init_timer() {
	// Set interval interrupt handler.
	register_handler(32, interval_timer_handler as u32);
}

#[no_mangle]
pub extern "C" fn handle_interval_timer() {
	// Send EOI
	outb(0x20, 0x20);

	unsafe {
		CLOCK += 1;
		if CLOCK % FREQ == 0 {
			write!(VGA, "\rclock: {}", CLOCK / FREQ).expect("clock");
		}
	}
}

use core::fmt::Write;

use crate::{
	arch::x86::interrupt_descriptor_table::{
		register_handler, InterruptStackFrame,
	},
	isr,
	vga::VGA,
	x86::common::outb,
};

static mut CLOCK: u32 = 0;

const FREQ: u32 = 18;

pub fn init_timer() {
	// Set interval interrupt handler.
	register_handler(isr!(32, handle_interval_timer));
}

#[no_mangle]
pub extern "C" fn handle_interval_timer(_: &InterruptStackFrame) {
	// Send EOI
	outb(0x20, 0x20);

	unsafe {
		CLOCK += 1;
		if CLOCK % FREQ == 0 {
			write!(VGA, "\rclock: {}", CLOCK / FREQ).expect("clock");
		}
	}
}

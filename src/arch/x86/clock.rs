use crate::{
	arch::x86::interrupt_descriptor_table::{
		register_handler, InterruptRequest,
	},
	isr,
	x86::common::outb,
};

static mut CLOCK: u32 = 0;

const FREQ: u32 = 18;

pub fn init_clock() {
	// Set interval interrupt handler.
	register_handler(isr!(32, handle_interval_timer));
}

pub extern "C" fn handle_interval_timer(_: &InterruptRequest) {
	// Send EOI
	outb(0x20, 0x20);

	unsafe {
		CLOCK += 1;
	}
}

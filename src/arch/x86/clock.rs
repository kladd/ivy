use core::sync::atomic::{AtomicU32, Ordering};

use crate::{
	arch::x86::interrupt_descriptor_table::{
		register_handler, InterruptRequest,
	},
	isr,
	x86::common::outb,
};

const FREQ: u32 = 18;

static CLOCK: AtomicU32 = AtomicU32::new(0);

pub fn init_clock() {
	// Set interval interrupt handler.
	register_handler(isr!(32, handle_interval_timer));
}

pub fn uptime_seconds() -> u32 {
	// 18.222 (repeating of course), so not accurate here really
	CLOCK.load(Ordering::Relaxed) / 18
}

pub extern "C" fn handle_interval_timer(_: &InterruptRequest) {
	// Send EOI
	outb(0x20, 0x20);
	CLOCK.fetch_add(1, Ordering::Relaxed);
}

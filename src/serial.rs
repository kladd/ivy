use core::cell::Cell;

use crate::arch::x86::outb;

pub const COM1: SerialPort = SerialPort {
	port: 0x3F8 as *mut u8,
	initialized: Cell::new(false),
};

pub struct SerialPort {
	port: *mut u8,
	initialized: Cell<bool>,
}

impl SerialPort {
	pub fn init(&self) {
		if self.initialized.get() {
			return;
		}

		unsafe {
			// TODO: Document.
			*self.port.offset(1isize) = 0x00;
			*self.port.offset(3isize) = 0x80;
			*self.port.offset(0isize) = 0x02;
			*self.port.offset(1isize) = 0x00;
			*self.port.offset(3isize) = 0x03;
			*self.port.offset(2isize) = 0xC7;
			*self.port.offset(4isize) = 0x0B;
		}
		self.initialized.set(true);
	}

	fn transmit_empty(&self) -> bool {
		unsafe { *self.port.offset(5isize) & 0x20 == 0 }
	}

	fn write_byte(&mut self, b: u8) -> core::fmt::Result {
		while self.transmit_empty() {}
		outb(self.port as u16, b);

		Ok(())
	}
}

impl ::core::fmt::Write for SerialPort {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for b in s.bytes() {
			self.write_byte(b)?;
		}

		Ok(())
	}
}

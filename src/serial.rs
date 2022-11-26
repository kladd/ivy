use core::{arch::asm, cell::Cell, fmt::Write};

pub const COM1: SerialPort = SerialPort {
	port: 0x3F8 as *mut u8,
	initialized: Cell::new(false),
};

pub struct SerialPort {
	port: *mut u8,
	initialized: Cell<bool>,
}

impl SerialPort {
	fn transmit_empty(&self) -> bool {
		unsafe { *self.port.offset(5isize) & 0x20 == 0 }
	}

	fn write_byte(&mut self, b: u8) -> core::fmt::Result {
		while self.transmit_empty() {}
		unsafe {
			outb(b, self.port as u16);
		}

		Ok(())
	}
}

impl Write for SerialPort {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for b in s.bytes() {
			self.write_byte(b)?;
		}

		Ok(())
	}
}

pub fn init_port(port: SerialPort) {
	if port.initialized.get() {
		return;
	}

	unsafe {
		// TODO: Document.
		*port.port.offset(1isize) = 0x00;
		*port.port.offset(3isize) = 0x80;
		*port.port.offset(0isize) = 0x02;
		*port.port.offset(1isize) = 0x00;
		*port.port.offset(3isize) = 0x03;
		*port.port.offset(2isize) = 0xC7;
		*port.port.offset(4isize) = 0x0B;
	}
	port.initialized.set(true);
}

/// x86 OUT instruction for byte operands.
unsafe fn outb(b: u8, port: u16) {
	// Output byte in al to I/O port address in dx.
	asm!("out dx, al", in("al") b, in("dx") port);
}

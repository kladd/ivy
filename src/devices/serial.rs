use core::fmt::Write;

use log::warn;

use crate::{
	arch::amd64::{inb, outb},
	devices::character::{Keycode, ReadCharacter, WriteCharacter},
	sync::{InitOnce, SpinLock},
};

#[derive(Debug)]
pub struct SerialPort(u16);

static COM1: InitOnce<SpinLock<SerialPort>> = InitOnce::new();

pub fn com1<'a>() -> &'a SpinLock<SerialPort> {
	COM1.get_or_init(|| {
		let serial = SerialPort(0x3F8);
		serial.init();
		SpinLock::new(serial)
	})
}

pub fn init_serial() {
	com1();
}

impl ReadCharacter for SerialPort {
	fn getc(&mut self) -> Option<Keycode> {
		match self.read_byte() {
			21 => Some(Keycode::Nak),
			13 => Some(Keycode::Newline),
			12 => Some(Keycode::FormFeed),
			8 => Some(Keycode::Backspace),
			b => Some(Keycode::Char(b as char)),
		}
	}
}

impl WriteCharacter for SerialPort {
	fn putc(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Char(c) => self.write_byte(c as u8).unwrap(),
			Keycode::Newline => self.write_byte('\n' as u8).unwrap(),
			Keycode::Backspace => {
				self.write_str("\x08 \x08").unwrap();
			}
			Keycode::FormFeed => {
				warn!("TODO: Form feed (0x0C) doesn't work for some reason")
			}
			code => warn!("{code:?} unimplemented"),
		};
	}
}

impl SerialPort {
	fn init(&self) {
		outb(self.0 + 1, 0x00);
		outb(self.0 + 3, 0x80);
		outb(self.0 + 0, 0x02);
		outb(self.0 + 1, 0x00);
		outb(self.0 + 3, 0x03);
		outb(self.0 + 2, 0xC7);
		outb(self.0 + 4, 0x0B);
	}

	pub fn read_byte(&self) -> u8 {
		self.await_recv();
		inb(self.0)
	}

	fn write_byte(&self, b: u8) -> core::fmt::Result {
		self.await_transmit();
		outb(self.0, b);
		Ok(())
	}

	fn await_recv(&self) {
		while inb(self.0 + 5) & 0x01 == 0 {}
	}

	fn await_transmit(&self) {
		while inb(self.0 + 5) & 0x20 == 0 {}
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

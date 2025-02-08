use alloc::string::String;
use core::fmt::Write;

use crate::{
	arch::amd64::hlt,
	devices::{
		character::{Keycode, ReadCharacter, WriteCharacter},
		keyboard::KBD,
		vga::vga0,
	},
	sync::{SpinLock, StaticPtr},
};

static TTY0: StaticPtr<SpinLock<Terminal>> = StaticPtr::new();

pub fn init() {
	TTY0.init(SpinLock::new(Terminal));
}

pub fn tty0() -> &'static mut SpinLock<Terminal> {
	TTY0.get()
}

pub struct Terminal;

impl Terminal {
	pub fn read_line(&mut self) -> String {
		let mut s = String::with_capacity(80);
		loop {
			match self.getc() {
				Some(Keycode::Backspace) => {
					if s.pop().is_some() {
						self.putc(Keycode::Backspace);
					}
				}
				Some(Keycode::Nak) => {
					while s.pop().is_some() {
						self.putc(Keycode::Backspace)
					}
				}
				Some(Keycode::Char(c)) => {
					s.write_char(c).unwrap();
					self.putc(Keycode::Char(c));
				}
				Some(Keycode::Newline) => {
					self.putc(Keycode::Newline);
					break;
				}
				Some(kc) => self.putc(kc),
				_ => {}
			}
			hlt();
		}
		s
	}
}

impl WriteCharacter for Terminal {
	fn putc(&mut self, keycode: Keycode) {
		vga0().lock().putc(keycode);
	}
}

impl ReadCharacter for Terminal {
	fn getc(&mut self) -> Option<Keycode> {
		unsafe { KBD.getc() }
	}
}

impl Write for Terminal {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		vga0().lock().write_str(s)
	}
}

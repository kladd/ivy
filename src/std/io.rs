use alloc::string::String;
use core::fmt::Write;

use crate::{
	arch::x86::halt,
	devices::character::{Keycode, ReadCharacter, WriteCharacter},
};

const KBD_BUFFER_SIZE: usize = 16;
const MAX_LINE_LEN: usize = 78;

pub struct Terminal<'a, R, W>
where
	R: ReadCharacter,
	W: WriteCharacter + Write,
{
	pub read: &'a mut R,
	pub write: &'a mut W,
}

impl<'a, R, W> Terminal<'a, R, W>
where
	R: ReadCharacter,
	W: WriteCharacter + Write,
{
	pub fn read_line(&mut self) -> String {
		let mut s = String::with_capacity(MAX_LINE_LEN);
		loop {
			match self.read.getc() {
				Some(Keycode::Newline) => {
					self.write.putc(Keycode::Newline);
					break;
				}
				Some(Keycode::Char(c)) => {
					self.write.putc(Keycode::Char(c));
					s.write_char(c).unwrap();
				}
				Some(Keycode::Backspace) => {
					if s.pop().is_some() {
						self.write.putc(Keycode::Backspace);
					}
				}
				Some(Keycode::FormFeed) => self.write.putc(Keycode::FormFeed),
				Some(Keycode::VerticalTab) => {
					self.write.putc(Keycode::VerticalTab)
				}
				Some(Keycode::Nak) => {
					while s.pop().is_some() {
						self.write.putc(Keycode::Backspace)
					}
				}
				_ => continue,
			}
			halt();
		}
		s
	}
}

impl<'a, R, W> Write for Terminal<'a, R, W>
where
	R: ReadCharacter,
	W: WriteCharacter + Write,
{
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		self.write.write_str(s)
	}
}

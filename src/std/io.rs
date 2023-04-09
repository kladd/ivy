use core::fmt::Write;

use crate::{
	arch::x86::halt,
	keyboard::{Keyboard, Keycode},
	std::string::String,
	vga::VideoMemory,
};

const KBD_BUFFER_SIZE: usize = 16;
const MAX_LINE_LEN: usize = 78;

pub struct Terminal<'a> {
	pub kbd: &'a mut Keyboard<KBD_BUFFER_SIZE>,
	pub vga: VideoMemory,
}

impl<'a> Terminal<'a> {
	pub fn read_line(&mut self) -> String {
		let mut s = String::new(MAX_LINE_LEN);
		loop {
			match self.kbd.getc() {
				Some(Keycode::Newline) => {
					self.vga.insert_newline().unwrap();
					break;
				}
				Some(Keycode::Char(c)) => {
					self.vga.write_char(c).unwrap();
					s.write_char(c).unwrap();
				}
				Some(Keycode::Backspace) => {
					if s.pop().is_some() {
						self.vga.backspace();
					}
				}
				Some(Keycode::FormFeed) => self.vga.form_feed(),
				Some(Keycode::VerticalTab) => self.vga.vertical_tab(),
				Some(Keycode::Nak) => {
					while s.pop().is_some() {
						self.vga.backspace()
					}
				}
				_ => continue,
			}
			halt();
		}
		s
	}
}

impl<'a> Write for Terminal<'a> {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		self.vga.write_str(s)
	}
}

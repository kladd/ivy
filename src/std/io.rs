use alloc::string::String;
use core::{fmt::Write, mem::MaybeUninit};

use crate::{
	arch::x86::halt,
	devices::{
		character::{Keycode, ReadCharacter, WriteCharacter},
		keyboard,
		keyboard::{Keyboard, KBD},
		serial::{SerialPort, COM1},
	},
	vga::{VideoMemory, VGA},
};

const KBD_BUFFER_SIZE: usize = 16;
const MAX_LINE_LEN: usize = 78;

pub type VideoTerminal =
	Terminal<'static, Keyboard<{ keyboard::BUFFER_SIZE }>, VideoMemory>;
pub type SerialTerminal = Terminal<'static, SerialPort, SerialPort>;

static mut VGA_TERM: MaybeUninit<VideoTerminal> = MaybeUninit::uninit();
static mut SERIAL_TERM: MaybeUninit<SerialTerminal> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct Terminal<'a, R, W>
where
	R: ReadCharacter,
	W: WriteCharacter + Write,
{
	pub read: &'a mut R,
	pub write: &'a mut W,
}

impl VideoTerminal {
	pub fn init() {
		unsafe {
			VGA_TERM.write(Terminal {
				read: &mut KBD,
				write: &mut VGA,
			})
		};
	}

	pub fn global_mut() -> &'static mut Self {
		unsafe { VGA_TERM.assume_init_mut() }
	}
}

impl SerialTerminal {
	pub fn init() {
		unsafe {
			SERIAL_TERM.write(Terminal {
				read: &mut COM1,
				write: &mut COM1,
			})
		};
	}

	pub fn global_mut() -> &'static mut Self {
		unsafe { SERIAL_TERM.assume_init_mut() }
	}
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

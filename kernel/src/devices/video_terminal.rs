use alloc::{string::String, vec::Vec};
use core::fmt::Write;

const COLS: usize = 80;
const ROWS: usize = 26;

use crate::{
	arch::amd64::hlt,
	devices::{
		character::{Keycode, ReadCharacter, WriteCharacter},
		keyboard,
		keyboard::Keyboard,
		video::Video,
	},
};

pub struct VideoTerminal<'a> {
	screen: Video<'a>,
	kbd: &'a mut Keyboard<{ keyboard::BUFFER_SIZE }>,
	cursor: Cursor,
	buf: [char; ROWS * COLS],
}

struct Cursor {
	row: usize,
	col: usize,
}

impl<'a> VideoTerminal<'a> {
	const MAX_LINE_LEN: usize = 78;
	const CURSOR: char = 177u8 as char;

	pub fn new(
		video: Video<'a>,
		kbd: &'a mut Keyboard<{ keyboard::BUFFER_SIZE }>,
	) -> Self {
		Self {
			screen: video,
			kbd,
			cursor: Cursor { row: 0, col: 0 },
			buf: [0u8 as char; ROWS * COLS],
		}
	}

	pub fn test(&mut self) {
		self.clear();
		self.screen.test();
	}

	pub fn read_line(&mut self) -> String {
		let mut s = String::with_capacity(Self::MAX_LINE_LEN);
		loop {
			match self.kbd.getc() {
				Some(Keycode::Newline) => {
					self.putc(Keycode::Newline);
					self.blit();
					break;
				}
				Some(Keycode::Char(c)) => {
					self.putc(Keycode::Char(c));
					s.write_char(c).unwrap();
				}
				Some(Keycode::Backspace) => {
					if s.pop().is_some() {
						self.putc(Keycode::Backspace);
					}
				}
				Some(Keycode::FormFeed) => self.putc(Keycode::FormFeed),
				Some(Keycode::VerticalTab) => self.putc(Keycode::VerticalTab),
				Some(Keycode::Nak) => {
					s.clear();
					self.putc(Keycode::Nak);
				}
				_ => continue,
			}
			self.blit();
			hlt();
		}
		s
	}

	pub fn clear(&mut self) {
		for row in 0..ROWS {
			for col in 0..COLS {
				self.buf[row * COLS + col] = 0 as char;
			}
		}
		self.cursor.col = 0;
		self.cursor.row = 0;
		self.update_cursor();
		self.blit();
	}

	fn update_cursor(&mut self) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = Self::CURSOR;
	}

	fn char(&mut self, c: char) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = c;
		// TODO: Soft wrap is probably not always desirable.
		self.cursor.row += (self.cursor.col + 1) / COLS;
		self.cursor.col = (self.cursor.col + 1) % COLS;
	}

	fn backspace(&mut self) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = 0 as char;
		if self.cursor.col > 0 {
			self.cursor.col -= 1;
		}
	}

	fn newline(&mut self) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = 0 as char;
		if self.cursor.row == ROWS - 1 {
			self.scroll();
			self.nak();
		} else {
			self.cursor.row += 1;
		}
		self.cursor.col = 0;
	}

	fn nak(&mut self) {
		for col in 0..=self.cursor.col {
			self.buf[self.cursor.row * COLS + col] = 0 as char;
		}
		self.cursor.col = 0;
	}

	fn scroll(&mut self) {
		for i in COLS..(ROWS * COLS) {
			self.buf[i - COLS] = self.buf[i];
		}
	}

	fn blit(&mut self) {
		for row in 0..ROWS {
			for col in 0..COLS {
				self.screen.glyph(self.buf[row * COLS + col], col, row);
			}
		}
	}
}

impl<'a> WriteCharacter for VideoTerminal<'a> {
	fn putc(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Newline => self.newline(),
			Keycode::Nak => self.nak(),
			// TODO:
			// Keycode::FormFeed => self.form_feed(),
			// Keycode::VerticalTab => self.vertical_tab(),
			Keycode::Backspace => self.backspace(),
			Keycode::Char(c) => self.char(c),
			Keycode::Null => {}
			Keycode::StartOfHeading => {}
			_ => {}
		}
		self.update_cursor();
	}
}

impl<'a> ReadCharacter for VideoTerminal<'a> {
	fn getc(&mut self) -> Option<Keycode> {
		self.kbd.getc()
	}
}

impl<'a> Write for VideoTerminal<'a> {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for c in s.chars() {
			match c {
				'\n' => self.putc(Keycode::Newline),
				_ => self.putc(Keycode::Char(c)),
			}
		}
		self.blit();
		Ok(())
	}
}

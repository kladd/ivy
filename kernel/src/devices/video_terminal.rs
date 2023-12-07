use alloc::{string::String, vec::Vec};
use core::fmt::Write;

const COLS: usize = 80;
const ROWS: usize = 26;

use crate::{
	arch::amd64::hlt,
	devices::{
		character::{Keycode, ReadCharacter, WriteCharacter},
		keyboard::KBD,
		video::vd0,
	},
	sync::StaticPtr,
};

static VDT0: StaticPtr<VideoTerminal> = StaticPtr::new();

pub fn init() {
	VDT0.init(VideoTerminal::new());
}

pub fn vdt0() -> &'static mut VideoTerminal {
	VDT0.get()
}

pub struct VideoTerminal {
	cursor: Cursor,
	buf: [char; ROWS * COLS],
}

struct Cursor {
	row: usize,
	col: usize,
}

impl VideoTerminal {
	const MAX_LINE_LEN: usize = 78;
	const CURSOR: char = 177u8 as char;

	pub fn new() -> Self {
		Self {
			cursor: Cursor { row: 0, col: 0 },
			buf: [0u8 as char; ROWS * COLS],
		}
	}

	pub fn test(&mut self) {
		self.clear();
		vd0().test();
	}

	pub fn read_line(&mut self) -> String {
		let mut s = String::with_capacity(Self::MAX_LINE_LEN);
		loop {
			match unsafe { KBD.getc() } {
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
				vd0().glyph(self.buf[row * COLS + col], col, row);
			}
		}
	}
}

impl WriteCharacter for VideoTerminal {
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

impl ReadCharacter for VideoTerminal {
	fn getc(&mut self) -> Option<Keycode> {
		unsafe { KBD.getc() }
	}
}

impl Write for VideoTerminal {
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

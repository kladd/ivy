use alloc::{string::String, vec::Vec};
use core::fmt::Write;

use log::debug;

const COLS: usize = 80;
const ROWS: usize = 25;

const LF: u8 = '\n' as u8;
const CR: u8 = '\r' as u8;
const TAB: u8 = '\t' as u8;

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
	VDT0.get().clear();
	VDT0.get().update_cursor();
	VDT0.get().blit();
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
	pub fn new() -> Self {
		Self {
			cursor: Cursor { row: 0, col: 0 },
			buf: [0u8 as char; ROWS * COLS],
		}
	}

	pub fn read_line(&mut self) -> String {
		let mut s = String::with_capacity(Self::MAX_LINE_LEN);
		loop {
			match unsafe { self.getc() } {
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
}

impl VideoTerminal {
	const MAX_LINE_LEN: usize = 78;
	const CURSOR: char = 177u8 as char;

	fn write_byte(&mut self, b: u8) {
		match b {
			LF => self.newline(),
			CR => self.carriage_return(),
			TAB => self.write_str("    ").unwrap(),
			_ => self.write_byte_visible(b),
		}
	}

	fn write_byte_visible(&mut self, b: u8) {
		let pos = self.pos();
		self.buf[pos] = b as char;
		self.advance();
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

	fn carriage_return(&mut self) {
		self.cursor.col = 0;
	}

	fn clear(&mut self) {
		for i in 0..(COLS * ROWS) {
			self.write_blank(i);
		}
		self.cursor.col = 0;
		self.cursor.row = 0;
	}

	fn form_feed(&mut self) {
		// Write the current line to line 0.
		let row_offset = row_to_cell(self.cursor.row);
		for i in 0..COLS {
			self.buf[i] = self.buf[row_offset + i];
		}

		// Write blank to everything else.
		for i in COLS..(COLS * ROWS) {
			self.write_blank(i);
		}

		// Retain the column, update the row.
		self.cursor.row = 0;
	}

	fn nak(&mut self) {
		for col in 0..=self.cursor.col {
			self.buf[self.cursor.row * COLS + col] = 0 as char;
		}
		self.cursor.col = 0;
	}

	fn backspace(&mut self) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = 0 as char;
		if self.cursor.col > 0 {
			self.cursor.col -= 1;
		}
	}

	fn start_of_heading(&mut self) {
		self.cursor.col = 0;
	}

	fn vertical_tab(&mut self) {
		let eol = row_to_cell(self.cursor.row + 1);
		for i in self.pos()..eol {
			self.write_blank(i);
		}
	}

	fn scroll(&mut self) {
		for i in COLS..(ROWS * COLS) {
			self.buf[i - COLS] = self.buf[i];
		}
	}

	fn update_cursor(&mut self) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = Self::CURSOR;
	}

	fn pos(&self) -> usize {
		self.cursor.row * COLS + self.cursor.col
	}

	fn advance(&mut self) {
		let next_col = self.cursor.col + 1;
		self.cursor.row += next_col / COLS;
		self.cursor.col = next_col % COLS;
	}

	fn char(&mut self, c: char) {
		self.buf[self.cursor.row * COLS + self.cursor.col] = c;
		// TODO: Soft wrap is probably not always desirable.
		self.cursor.row += (self.cursor.col + 1) / COLS;
		self.cursor.col = (self.cursor.col + 1) % COLS;
	}

	fn blit(&mut self) {
		for row in 0..ROWS {
			for col in 0..COLS {
				vd0().glyph(self.buf[row * COLS + col], col, row);
			}
		}
	}

	fn write_blank(&mut self, idx: usize) {
		self.buf[idx] = 0 as char;
	}
}

fn row_to_cell(row: usize) -> usize {
	row * COLS
}

impl WriteCharacter for VideoTerminal {
	fn putc(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Newline => self.newline(),
			Keycode::Nak => self.nak(),
			Keycode::FormFeed => self.form_feed(),
			Keycode::VerticalTab => self.vertical_tab(),
			Keycode::Backspace => self.backspace(),
			Keycode::Char(c) => self.char(c),
			Keycode::Null => {}
			Keycode::StartOfHeading => self.start_of_heading(),
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
		for c in s.bytes() {
			self.write_byte(c);
		}
		self.update_cursor();
		self.blit();
		Ok(())
	}
}

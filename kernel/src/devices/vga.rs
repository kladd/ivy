use core::fmt::Write;

use log::trace;

use crate::{
	arch::amd64::outb,
	devices::character::{Keycode, WriteCharacter},
	mem::PhysicalAddress,
	sync::{SpinLock, StaticPtr},
};

const ROWS: usize = 25;
const COLS: usize = 80;

const BG: Color = Color::Blue;
const FG: Color = Color::White;

const LF: u8 = '\n' as u8;
const CR: u8 = '\r' as u8;

static VGA0: StaticPtr<SpinLock<VideoMemory>> = StaticPtr::new();

pub fn init() {
	VGA0.init(SpinLock::new(VideoMemory {
		row: 0,
		col: 0,
		addr: PhysicalAddress(0xB8000).to_virtual(),
	}));
	let mut guard = VGA0.get().lock();
	guard.clear();
	guard.update_cursor();
}

pub fn vga0() -> &'static SpinLock<VideoMemory> {
	VGA0.get()
}

pub struct VideoMemory {
	row: usize,
	col: usize,
	addr: *mut u16,
}

#[repr(u16)]
enum Color {
	Black,
	Blue,
	White = 0xF,
}

impl VideoMemory {
	fn write_byte(&mut self, b: u8) {
		match b {
			LF => self.newline(),
			CR => self.carriage_return(),
			_ => self.write_byte_visible(b),
		}
	}

	fn write_byte_visible(&mut self, b: u8) {
		let pos = self.pos();
		unsafe {
			*self.addr.offset(self.pos() as isize) = to_cell(b, FG, BG);
		}
		self.advance();
	}

	fn newline(&mut self) {
		if self.row < ROWS - 1 {
			self.row += 1;
		} else {
			self.scroll();
			self.vertical_tab();
			self.nak();
		}
		self.col = 0;
	}

	fn carriage_return(&mut self) {
		self.col = 0;
	}

	fn clear(&mut self) {
		for i in 0..(COLS * ROWS) as isize {
			self.write_blank(i);
		}
		self.row = 0;
		self.col = 0;
	}

	fn form_feed(&mut self) {
		// Write the current line to line 0.
		let row_offset = row_to_cell(self.row) as isize;
		for i in 0..COLS as isize {
			let borrow_checker = self.read_offset(row_offset + i);
			self.write_offset(i, borrow_checker);
		}

		// Write blank to everything else.
		for i in COLS as isize..(COLS * ROWS) as isize {
			self.write_blank(i);
		}

		// Retain the column, update the row.
		self.row = 0;
	}

	fn nak(&mut self) {
		let line = row_to_cell(self.row) as isize;
		let cursor = line + self.col as isize;
		for i in line..cursor {
			self.write_blank(i);
		}

		self.col = 0;
	}

	fn backspace(&mut self) {
		if self.col > 0 {
			self.col -= 1;
			self.write_blank(self.pos() as isize)
		}
	}

	fn start_of_heading(&mut self) {
		self.col = 0;
	}

	fn vertical_tab(&mut self) {
		let cursor = self.pos() as isize;
		let eol = row_to_cell(self.row + 1) as isize;
		for i in cursor..eol {
			self.write_blank(i);
		}
	}

	fn scroll(&mut self) {
		for i in COLS..(ROWS * COLS) {
			let borrow_checker = self.read_offset(i as isize);
			self.write_offset((i - COLS) as isize, borrow_checker);
		}
	}

	fn update_cursor(&mut self) {
		let cursor = self.pos();

		outb(0x3D4, 0x0F);
		outb(0x3D5, (cursor & 0xFF) as u8);
		outb(0x3D4, 0x0E);
		outb(0x3D5, ((cursor >> 8) & 0xFF) as u8);
	}

	fn pos(&self) -> usize {
		self.row * COLS + self.col
	}

	fn advance(&mut self) {
		let next_col = self.col + 1;
		self.row += next_col / COLS;
		self.col = next_col % COLS;
	}

	fn write_offset(&mut self, offset: isize, val: u16) {
		unsafe { *self.addr.offset(offset) = val };
	}

	fn read_offset(&mut self, offset: isize) -> u16 {
		unsafe { *self.addr.offset(offset) }
	}

	fn write_blank(&mut self, offset: isize) {
		self.write_offset(offset, to_cell(0x20u16, FG, BG))
	}
}

impl WriteCharacter for VideoMemory {
	fn putc(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Backspace => self.backspace(),
			Keycode::Char(c) => self.write_byte(c as u8),
			Keycode::FormFeed => self.form_feed(),
			Keycode::Nak => self.nak(),
			Keycode::Newline => self.newline(),
			Keycode::Null => {}
			Keycode::StartOfHeading => self.start_of_heading(),
			Keycode::VerticalTab => self.vertical_tab(),
		}
		self.update_cursor();
	}
}

impl Write for VideoMemory {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for byte in s.bytes() {
			self.write_byte(byte);
		}
		self.update_cursor();

		Ok(())
	}
}

fn to_cell<T: Into<u16>>(c: T, fg: Color, bg: Color) -> u16 {
	c.into() | ((fg as u16) | (bg as u16) << 4) << 8
}

fn row_to_cell(row: usize) -> usize {
	row * COLS
}

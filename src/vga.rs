use core::{cell::UnsafeCell, fmt::Write};

use crate::{
	arch::x86::outb,
	devices::character::{Keycode, WriteCharacter},
};

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const ROWS: usize = 25;
const COLS: usize = 80;

const LINE_FEED: u8 = '\n' as u8;
const CARRIAGE_RETURN: u8 = '\r' as u8;

const BLACK: u8 = 0x0;
const BLUE: u8 = 0x1;
const WHITE: u8 = 0xF;

pub static mut VGA: VideoMemory = VideoMemory::get();

static mut CARRIER: Carrier = Carrier {
	row: UnsafeCell::new(0),
	col: UnsafeCell::new(0),
	color: UnsafeCell::new(WHITE | BLUE << 4),
};

struct Carrier {
	row: UnsafeCell<usize>,
	col: UnsafeCell<usize>,
	color: UnsafeCell<u8>,
}

impl Carrier {
	fn pos(&self) -> usize {
		unsafe { *self.row.get() * COLS + *self.col.get() }
	}

	fn cell(&self, c: u8) -> u16 {
		unsafe { (*self.color.get() as u16) << 8 | c as u16 }
	}

	fn row(&self) -> usize {
		unsafe { *self.row.get() }
	}

	fn set_row(&self, to: usize) {
		unsafe { *self.row.get() = to };
	}

	fn inc_row(&self, by: usize) {
		unsafe { *self.row.get() += by };
	}

	fn col(&self) -> usize {
		unsafe { *self.col.get() }
	}

	fn set_col(&self, to: usize) {
		unsafe { *self.col.get() = to };
	}

	fn write_cell(&self, c: u8) {
		unsafe {
			*VIDEO_MEMORY.offset(self.pos() as isize) = self.cell(c);
		}
	}
}

pub struct VideoMemory;

impl WriteCharacter for VideoMemory {
	fn putc(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Newline => self.insert_newline().unwrap(),
			Keycode::Nak => self.nak(),
			Keycode::FormFeed => self.form_feed(),
			Keycode::VerticalTab => self.vertical_tab(),
			Keycode::Backspace => self.backspace(),
			Keycode::Char(c) => self.write_byte(c as u8).unwrap(),
			Keycode::Null => {}
			Keycode::StartOfHeading => {}
		}
		self.update_cursor()
	}
}

impl VideoMemory {
	pub const fn get() -> Self {
		Self
	}

	fn write_byte(&mut self, byte: u8) -> core::fmt::Result {
		match byte {
			LINE_FEED => self.insert_newline(),
			CARRIAGE_RETURN => self.insert_carriage_return(),
			_ => self.write_byte_visible(byte),
		}
	}

	pub fn insert_newline(&self) -> core::fmt::Result {
		if unsafe { CARRIER.row() } == ROWS - 1 {
			self.scroll();
			self.vertical_tab();
			self.nak();
		} else {
			unsafe { CARRIER.set_row(CARRIER.row() + 1) };
		}
		unsafe { CARRIER.set_col(0) };
		self.update_cursor();

		Ok(())
	}

	fn insert_carriage_return(&self) -> core::fmt::Result {
		unsafe { CARRIER.set_col(0) };
		self.update_cursor();

		Ok(())
	}

	fn write_byte_visible(&self, byte: u8) -> core::fmt::Result {
		unsafe {
			CARRIER.write_cell(byte);
			CARRIER.inc_row((CARRIER.col() + 1) / COLS);
			CARRIER.set_col((CARRIER.col() + 1) % COLS);
		}

		Ok(())
	}

	pub fn clear_screen(&self) {
		for i in 0..(COLS * ROWS) as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = CARRIER.cell(0x20);
			}
		}
		unsafe {
			CARRIER.set_row(0);
			CARRIER.set_col(0);
		}

		self.update_cursor();
	}

	pub fn disable_cursor(&self) {
		outb(0x3D4, 0x0A);
		outb(0x3D5, 0x20);
	}

	pub fn form_feed(&self) {
		// Write the cursor line to line 0.
		for i in 0..COLS as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i) =
					*VIDEO_MEMORY.offset((CARRIER.row() * COLS) as isize + i)
			}
		}
		// Write blank to everything else.
		for i in COLS as isize..(COLS * ROWS) as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i) = CARRIER.cell(0x20);
			}
		}
		// Update the row of the cursor but retain the column.
		unsafe { CARRIER.set_row(0) };
		self.update_cursor();
	}

	pub fn nak(&self) {
		let line_start = unsafe { CARRIER.row() } * COLS;
		let cursor = line_start + unsafe { CARRIER.col() };

		for i in line_start..cursor {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = CARRIER.cell(0x20);
			}
		}
		unsafe {
			CARRIER.set_col(0);
		}

		self.update_cursor();
	}

	pub fn backspace(&self) {
		let col = unsafe { CARRIER.col() };
		if col > 0 {
			unsafe {
				CARRIER.set_col(col - 1);
				CARRIER.write_cell(0x20);
			}
			self.update_cursor();
		}
	}

	pub fn start_of_heading(&self) {
		unsafe { CARRIER.set_col(0) };
		self.update_cursor();
	}

	pub fn vertical_tab(&self) {
		let cursor = unsafe { CARRIER.pos() };
		let eol = (unsafe { CARRIER.row() } + 1) * COLS;

		for i in cursor..eol {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = CARRIER.cell(0x20);
			}
		}
	}

	fn scroll(&self) {
		for i in COLS..(ROWS * COLS) {
			unsafe {
				*VIDEO_MEMORY.offset((i - COLS) as isize) =
					*VIDEO_MEMORY.offset(i as isize);
			}
		}
	}

	fn update_cursor(&self) {
		let cursor = unsafe { CARRIER.pos() };

		outb(0x3D4, 0x0F);
		outb(0x3D5, (cursor & 0xFF) as u8);
		outb(0x3D4, 0x0E);
		outb(0x3D5, ((cursor >> 8) & 0xFF) as u8);
	}
}

impl Write for VideoMemory {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for byte in s.bytes() {
			self.write_byte(byte)?;
		}
		self.update_cursor();

		Ok(())
	}
}

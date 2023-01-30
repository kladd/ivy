use core::fmt::Write;

use crate::x86::common::outb;

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const ROWS: usize = 25;
const COLS: usize = 80;

const LINE_FEED: u8 = '\n' as u8;
const CARRIAGE_RETURN: u8 = '\r' as u8;

pub static mut VGA: VideoMemory = VideoMemory {
	row: 0,
	col: 0,
	color: WHITE | BLUE << 4,
};

const BLACK: u8 = 0x0;
const BLUE: u8 = 0x1;
const WHITE: u8 = 0xF;

pub struct VideoMemory {
	row: usize,
	col: usize,
	color: u8,
}

impl VideoMemory {
	fn cell(&self, c: u8) -> u16 {
		(self.color as u16) << 8 | c as u16
	}

	fn write_byte(&mut self, byte: u8) -> core::fmt::Result {
		match byte {
			LINE_FEED => self.insert_newline(),
			CARRIAGE_RETURN => self.insert_carriage_return(),
			_ => self.write_byte_visible(byte),
		}
	}

	fn insert_newline(&mut self) -> core::fmt::Result {
		self.row += 1;
		self.col = 0;
		self.set_cursor();

		Ok(())
	}

	fn insert_carriage_return(&mut self) -> core::fmt::Result {
		self.col = 0;
		self.set_cursor();

		Ok(())
	}

	fn write_byte_visible(&mut self, byte: u8) -> core::fmt::Result {
		let cursor = self.row * COLS + self.col;
		unsafe {
			*VIDEO_MEMORY.offset(cursor as isize) = self.cell(byte);
		}

		self.row += (self.col + 1) / COLS;
		self.col = (self.col + 1) % COLS;

		Ok(())
	}

	pub fn clear_screen(&mut self) {
		for i in 0..(COLS * ROWS) as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i) = self.cell(0x20);
			}
		}

		self.row = 0;
		self.col = 0;
		self.set_cursor();
	}

	pub fn disable_cursor(&self) {
		outb(0x3D4, 0x0A);
		outb(0x3D5, 0x20);
	}

	pub fn form_feed(&mut self) {
		self.clear_screen();
	}

	pub fn nack(&mut self) {
		let line_start = self.row * COLS;
		let cursor = line_start + self.col;
		for i in line_start..cursor {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = self.cell(0x20);
			}
		}

		self.col = 0;
		self.set_cursor();
	}

	pub fn backspace(&mut self) {
		if self.col > 0 {
			self.col -= 1;
			self.set_cursor();
		}

		let cursor = self.row * COLS + self.col;
		unsafe {
			*VIDEO_MEMORY.offset(cursor as isize) = self.cell(0x20);
		}
	}

	pub fn start_of_heading(&mut self) {
		self.col = 0;
		self.set_cursor();
	}

	pub fn vertical_tab(&mut self) {
		let cursor = self.row * COLS + self.col;
		let eol = (self.row + 1) * COLS;

		for i in cursor..eol {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = self.cell(0x20);
			}
		}
	}

	fn set_cursor(&mut self) {
		let cursor = self.row * COLS + self.col;

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
		self.set_cursor();

		Ok(())
	}
}

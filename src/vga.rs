use core::fmt::Write;

use crate::x86::common::outb;

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const ROWS: usize = 25;
const COLS: usize = 80;

const LINE_FEED: u8 = '\n' as u8;
const CARRIAGE_RETURN: u8 = '\r' as u8;

const BLACK: u8 = 0x0;
const BLUE: u8 = 0x1;
const WHITE: u8 = 0xF;

static mut ROW: usize = 0;
static mut COL: usize = 0;
static mut COLOR: u8 = WHITE | BLUE << 4;

pub struct VideoMemory;

impl VideoMemory {
	pub fn get() -> Self {
		Self
	}

	fn cell(&self, c: u8) -> u16 {
		unsafe { (COLOR as u16) << 8 | c as u16 }
	}

	fn write_byte(&mut self, byte: u8) -> core::fmt::Result {
		match byte {
			LINE_FEED => self.insert_newline(),
			CARRIAGE_RETURN => self.insert_carriage_return(),
			_ => self.write_byte_visible(byte),
		}
	}

	fn insert_newline(&self) -> core::fmt::Result {
		unsafe {
			ROW += 1;
			COL = 0;
		}
		self.set_cursor();

		Ok(())
	}

	fn insert_carriage_return(&self) -> core::fmt::Result {
		unsafe { COL = 0 }
		self.set_cursor();

		Ok(())
	}

	fn write_byte_visible(&self, byte: u8) -> core::fmt::Result {
		unsafe {
			let cursor = ROW * COLS + COL;
			*VIDEO_MEMORY.offset(cursor as isize) = self.cell(byte);
			ROW += (COL + 1) / COLS;
			COL = (COL + 1) % COLS;
		}

		Ok(())
	}

	pub fn clear_screen(&self) {
		for i in 0..(COLS * ROWS) as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i) = self.cell(0x20);
			}
		}

		unsafe {
			ROW = 0;
			COL = 0;
		}

		self.set_cursor();
	}

	pub fn disable_cursor(&self) {
		outb(0x3D4, 0x0A);
		outb(0x3D5, 0x20);
	}

	pub fn form_feed(&self) {
		self.clear_screen();
	}

	pub fn nack(&self) {
		let line_start = unsafe { ROW * COLS };
		let cursor = unsafe { line_start + COL };
		for i in line_start..cursor {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = self.cell(0x20);
			}
		}

		unsafe { COL = 0 };
		self.set_cursor();
	}

	pub fn backspace(&self) {
		unsafe {
			if COL > 0 {
				COL -= 1;
				self.set_cursor();
			}

			let cursor = ROW * COLS + COL;
			*VIDEO_MEMORY.offset(cursor as isize) = self.cell(0x20);
		}
	}

	pub fn start_of_heading(&self) {
		unsafe { COL = 0 };
		self.set_cursor();
	}

	pub fn vertical_tab(&self) {
		let cursor = unsafe { ROW * COLS + COL };
		let eol = unsafe { (ROW + 1) * COLS };

		for i in cursor..eol {
			unsafe {
				*VIDEO_MEMORY.offset(i as isize) = self.cell(0x20);
			}
		}
	}

	fn set_cursor(&self) {
		let cursor = unsafe { ROW * COLS + COL };

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

use core::fmt::Write;

use crate::x86::common::outb;

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const ROWS: usize = 25;
const COLS: usize = 80;
const WHITE_ON_BLACK: u16 = 0x000F;

const LINE_FEED: u8 = '\n' as u8;
const CARRIAGE_RETURN: u8 = '\r' as u8;

pub static mut VGA: VideoMemory = VideoMemory { row: 0, col: 0 };

pub struct VideoMemory {
	row: usize,
	col: usize,
}

impl VideoMemory {
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

		Ok(())
	}

	fn insert_carriage_return(&mut self) -> core::fmt::Result {
		self.col = 0;

		Ok(())
	}

	fn write_byte_visible(&mut self, byte: u8) -> core::fmt::Result {
		let cursor = self.row * COLS + self.col;
		unsafe {
			*VIDEO_MEMORY.offset(cursor as isize) =
				WHITE_ON_BLACK << 8 | byte as u16;
		}

		self.row += (self.col + 1) / COLS;
		self.col = (self.col + 1) % COLS;

		Ok(())
	}

	pub fn clear_screen(&mut self) {
		for i in 0..(COLS * ROWS) as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i) = 0x0020;
			}
		}

		self.row = 0;
		self.col = 0;
	}

	pub fn disable_cursor(&self) {
		outb(0x3D4, 0x0A);
		outb(0x3D5, 0x20);
	}
}

impl Write for VideoMemory {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for byte in s.bytes() {
			self.write_byte(byte)?;
		}

		Ok(())
	}
}

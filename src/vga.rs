use core::{cell::Cell, fmt::Write};

use crate::x86::common::outb;

const VIDEO_MEMORY: *mut u16 = 0xB8000 as *mut u16;
const ROWS: usize = 25;
const COLS: usize = 80;
const WHITE_ON_BLACK: u16 = 0x000F;

const NEWLINE: u8 = '\n' as u8;

pub const VGA: VideoMemory = VideoMemory {
	row: Cell::new(0),
	col: Cell::new(0),
};

pub struct VideoMemory {
	row: Cell<usize>,
	col: Cell<usize>,
}

impl VideoMemory {
	fn write_byte(&mut self, byte: u8) -> core::fmt::Result {
		match byte {
			NEWLINE => self.insert_newline(),
			_ => self.write_byte_visible(byte),
		}
	}

	fn insert_newline(&mut self) -> core::fmt::Result {
		self.row.set(self.row.get() + 1);
		self.col.set(0);

		Ok(())
	}

	fn write_byte_visible(&mut self, byte: u8) -> core::fmt::Result {
		let cursor = self.row.get() * COLS + self.col.get();
		unsafe {
			*VIDEO_MEMORY.offset(cursor as isize) =
				WHITE_ON_BLACK << 8 | byte as u16;
		}

		self.row.set(self.row.get() + ((self.col.get() + 1) / COLS));
		self.col.set((self.col.get() + 1) % COLS);

		Ok(())
	}

	pub fn clear_screen(&self) {
		for i in 0..(COLS * ROWS) as isize {
			unsafe {
				*VIDEO_MEMORY.offset(i) = 0x0020;
			}
		}

		self.row.set(0);
		self.col.set(0);
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

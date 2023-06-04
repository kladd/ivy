use core::slice;

use crate::font::PSF2Font;

pub struct Video<'font>(&'static mut [u32], &'font PSF2Font);

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);

const A: u64 = 0xCCCCCCFC_CCCCCC78;

impl<'font> Video<'font> {
	const WIDTH: usize = 1024;
	const HEIGHT: usize = 768;
	const BG: Color = Color(0xFF, 0xFF, 0xFF);
	const FG: Color = Color(0x00, 0x00, 0x00);

	pub fn new(addr: *mut u32, font: &'font PSF2Font) -> Self {
		Self(
			unsafe {
				slice::from_raw_parts_mut(addr, Self::WIDTH * Self::HEIGHT)
			},
			font,
		)
	}

	pub fn test(&mut self) {
		for (i, pixel) in self.0.iter_mut().enumerate() {
			*pixel = Self::BG.into();
		}

		for i in 0..16 {
			for j in 0..16 {
				self.glyph((i * 16 + j) as u8 as char, j + 10, i + 5);
			}
		}
	}

	pub fn blank(&mut self, x: usize, y: usize) {
		let glyph_height = self.1.header.height as usize;
		let glyph_width = self.1.header.width as usize;

		for row in 0..glyph_height {
			for col in 0..glyph_width {
				let pos = (row + y * glyph_height) * Self::WIDTH
					+ col + (x * glyph_width);
				self.0[pos] = Self::BG.into();
			}
		}
	}

	pub fn glyph(&mut self, c: char, x: usize, y: usize) {
		let glyph = &self.1.data[c as usize];
		let glyph_height = self.1.header.height as usize;
		let glyph_width = self.1.header.width as usize;
		let bytes_per_row =
			(self.1.header.glyph_sz / self.1.header.height) as usize;

		// TODO: This is hacky as all get-out.
		for row in 0..glyph_height {
			for col in 0..glyph_width {
				let byte =
					glyph[row * bytes_per_row + (col / u8::BITS as usize)];
				let glyph_offset = (15 - col) % 8;

				let pos = (row + y * glyph_height) * Self::WIDTH
					+ col + (x * glyph_width);

				if (byte >> glyph_offset) & 1 != 0 {
					self.0[pos] = Self::FG.into();
				} else {
					self.0[pos] = Self::BG.into();
				}
			}
		}
	}
}

impl Into<u32> for Color {
	fn into(self) -> u32 {
		(self.0 as u32) << 16 | (self.1 as u32) << 8 | (self.2 as u32)
	}
}

use core::slice;

pub struct Video<'fb>(&'fb mut [u32]);

#[derive(Copy, Clone)]
pub struct Color(u8, u8, u8);

const A: u64 = 0xCCCCCCFC_CCCCCC78;

impl<'fb> Video<'fb> {
	const WIDTH: usize = 1024;
	const HEIGHT: usize = 768;
	const BG: Color = Color(0xFF, 0xFF, 0xFF);
	const FG: Color = Color(0x00, 0x00, 0x00);

	pub fn new(addr: usize) -> Self {
		Self(unsafe {
			slice::from_raw_parts_mut(
				addr as *mut u32,
				Self::WIDTH * Self::HEIGHT,
			)
		})
	}

	pub fn test(&mut self) {
		for (i, pixel) in self.0.iter_mut().enumerate() {
			*pixel = Self::BG.into();
		}

		for i in 0..5 {
			self.glyph(A, i, i);
		}
	}

	fn glyph(&mut self, map: u64, x: usize, y: usize) {
		for row in 0..8 {
			for col in 0..8 {
				// font height: 8, line height: 12
				let pos = ((4 + row) + y * 12) * Self::WIDTH + col + (x * 8);
				if (map >> ((row * 8 + col) as u64)) & 1 != 0 {
					self.0[pos] = Self::FG.into();
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

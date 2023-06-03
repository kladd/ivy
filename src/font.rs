use log::debug;

#[repr(C)]
#[derive(Debug)]
pub struct PSF2Header {
	magic: u32,
	version: u32,
	header_sz: u32,
	flags: u32,
	num_glyphs: u32,
	pub glyph_sz: u32,
	pub height: u32,
	pub width: u32,
}

#[repr(C)]
pub struct PSF2Font {
	pub header: PSF2Header,
	// TODO: hard coded font metrics.
	pub data: [[u8; 44]; 256],
}

impl PSF2Font {
	pub fn debug(&self, ascii: char) {
		let glyph = self.data[ascii as usize];

		for i in (0..glyph.len()).step_by(2) {
			debug!("{:08b}{:08b}", glyph[i], glyph[i + 1]);
		}
	}
}

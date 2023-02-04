static mut PLACEMENT_ADDR: u32 = 0x200000;
static mut MAX_ADDR: u32 = 0x400000;

pub fn kmalloc(size: usize) -> u32 {
	unsafe {
		let placement = PLACEMENT_ADDR;
		let next_placement = placement + size as u32;

		if next_placement < MAX_ADDR {
			PLACEMENT_ADDR = next_placement;
			placement
		} else {
			0
		}
	}
}

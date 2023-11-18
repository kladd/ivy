use log::{debug, trace};

use crate::mem::{PhysicalAddress, PAGE_SIZE};

#[derive(Debug)]
pub struct FrameAllocator {
	placement: usize,
	max: usize,
}

impl FrameAllocator {
	pub fn new(placement: usize, size: usize) -> Self {
		let aligned = placement & !(PAGE_SIZE - 1);
		kdbg!(Self {
			placement: aligned,
			max: aligned + size,
		})
	}

	pub fn alloc(&mut self) -> PhysicalAddress {
		let placement = self.placement;
		let next_placement = placement + PAGE_SIZE;

		debug!("placement: {placement:016X}, max: {:016X}", self.max);
		let page = if next_placement < self.max {
			self.placement = next_placement;
			PhysicalAddress(self.placement)
		} else {
			PhysicalAddress(0)
		};

		trace!("falloc({page:016X?}, {:016X})", PAGE_SIZE);

		page
	}
}

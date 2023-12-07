use log::{debug, trace};

use crate::{
	mem::{PhysicalAddress, PAGE_SIZE},
	sync::{SpinLock, StaticPtr},
};

static FRAME_ALLOCATOR: StaticPtr<SpinLock<FrameAllocator>> = StaticPtr::new();

#[derive(Debug)]
pub struct FrameAllocator {
	placement: usize,
	max: usize,
}

pub fn init(placement: usize, size: usize) {
	FRAME_ALLOCATOR.init(SpinLock::new(FrameAllocator::new(placement, size)));
}

pub fn current_mut() -> &'static SpinLock<FrameAllocator> {
	FRAME_ALLOCATOR.get()
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

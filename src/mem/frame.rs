use core::cell::OnceCell;

use log::{debug, info, trace};

use crate::{
	arch::amd64::vmem::PageTable,
	mem::{PhysicalAddress, PAGE_SIZE},
	sync::SpinLock,
};

pub struct FrameAllocator {
	placement: usize,
	max: usize,
}

static FRAME_ALLOCATOR: SpinLock<OnceCell<FrameAllocator>> =
	SpinLock::new(OnceCell::new());

impl FrameAllocator {
	pub fn init(placement: usize, size: usize) {
		let guard = FRAME_ALLOCATOR.lock();
		let f = guard.get_or_init(|| {
			let aligned = placement & !(PAGE_SIZE - 1);
			Self {
				placement: aligned,
				max: aligned + size,
			}
		});
		info!("FrameAllocator(0x{:016X} - 0x{:016X})", f.placement, f.max);
	}

	pub fn alloc() -> PhysicalAddress {
		let mut guard = FRAME_ALLOCATOR.lock();
		let allocator =
			guard.get_mut().expect("Frame allocator not initialized.");

		let placement = allocator.placement;
		let next_placement = placement + PAGE_SIZE;

		debug!("placement: {placement:016X}, max: {:016X}", allocator.max);
		let page = if next_placement < allocator.max {
			allocator.placement = next_placement;
			PhysicalAddress(allocator.placement)
		} else {
			PhysicalAddress(0)
		};

		trace!("falloc({page:016X?}, {:016X})", PAGE_SIZE);

		page
	}
}

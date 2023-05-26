use core::alloc::{GlobalAlloc, Layout};

use log::{debug, trace};

use crate::sync::SpinLock;

struct KernelAllocator {
	placement: usize,
	max: usize,
}

#[global_allocator]
static KERNEL_ALLOCATOR: SpinLock<KernelAllocator> =
	SpinLock::new(KernelAllocator {
		placement: !0,
		max: !0,
	});

impl SpinLock<KernelAllocator> {
	pub fn init(&self, placement: usize, max: usize) {
		debug!("init(0x{placement:016X})");
		let mut guard = self.lock();
		guard.max = max;
		guard.placement = placement;
	}
}

unsafe impl GlobalAlloc for SpinLock<KernelAllocator> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let mut guard = self.lock();
		let align = layout.align();
		guard.placement = (guard.placement & !(align - 1)) + align;

		let placement = guard.placement;
		let next_placement = placement + layout.size();

		let ptr = if next_placement < guard.max {
			guard.placement = next_placement;
			guard.placement
		} else {
			0
		};

		trace!("alloc(0x{ptr:016X}, {})", layout.size());

		ptr as *mut u8
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		trace!("leak(0x{ptr:016X?}, {})", layout.size());
	}
}

pub fn init_kalloc(placement: usize, size: usize) {
	KERNEL_ALLOCATOR.init((placement & !4095) + 4096, placement + size);
}

pub fn kmalloc(size: usize, align: usize) -> usize {
	unsafe {
		KERNEL_ALLOCATOR.alloc(Layout::from_size_align(size, align).unwrap())
			as usize
	}
}

use core::alloc::{GlobalAlloc, Layout};

use log::{debug, info, trace};

use crate::sync::SpinLock;

pub struct KernelAllocator {
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
		let mut guard = self.lock();
		guard.max = max;
		guard.placement = placement;
		info!("KernelAllocator(0x{:016X} - 0x{:016X})", placement, max);
	}
}

unsafe impl GlobalAlloc for SpinLock<KernelAllocator> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let mut guard = self.lock();
		let align = layout.align();
		guard.placement = kdbg!((guard.placement & !(align - 1)) + align);

		let placement = guard.placement;
		let next_placement = placement + layout.size();

		let ptr = if next_placement < kdbg!(guard.max) {
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

impl KernelAllocator {
	pub fn init(placement: usize, size: usize) {
		let aligned = (placement & !(0x200000 - 1)) + 0x200000;
		KERNEL_ALLOCATOR.init(aligned, aligned + size);
	}
}

pub fn kmalloc(size: usize, align: usize) -> usize {
	unsafe {
		KERNEL_ALLOCATOR.alloc(Layout::from_size_align(size, align).unwrap())
			as usize
	}
}

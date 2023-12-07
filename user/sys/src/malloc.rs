use core::alloc::{GlobalAlloc, Layout};

use crate::{
	sync::SpinLock,
	syscall::{brk, debug_long},
	PAGE_SIZE,
};

pub struct Allocator {
	placement: usize,
	max: usize,
}

impl Allocator {
	pub fn init(&mut self) {
		self.placement = brk(PAGE_SIZE);
		self.max = self.placement + PAGE_SIZE;
	}

	pub const fn new() -> Self {
		Self {
			placement: 0,
			max: 0,
		}
	}
}

unsafe impl GlobalAlloc for SpinLock<Allocator> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let mut guard = self.lock();
		let align = layout.align();
		guard.placement = (guard.placement & !(align - 1)) + align;

		let placement = guard.placement;
		let next_placement = placement + layout.size();

		let ptr = if next_placement < guard.max {
			guard.placement = next_placement;
			placement
		} else {
			0
		};

		// trace!("alloc(0x{ptr:016X}, {})", layout.size());

		ptr as *mut u8
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		// trace!("leak(0x{ptr:016X?}, {})", layout.size());
	}
}

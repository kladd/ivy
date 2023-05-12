use core::alloc::{GlobalAlloc, Layout};

use log::trace;

static mut PLACEMENT_ADDR: u32 = 0x200000;
static mut MAX_ADDR: u32 = 0x400000;

const PAGE_SIZE: u32 = 0x1000;

pub fn kmalloc_aligned(size: usize) -> u32 {
	kmalloc(size, PAGE_SIZE)
}

pub fn kmalloc(size: usize, alignment: u32) -> u32 {
	unsafe { PLACEMENT_ADDR = (PLACEMENT_ADDR & !(alignment - 1)) + alignment };
	let ptr = unsafe {
		let placement = PLACEMENT_ADDR;
		let next_placement = placement + size as u32;
		trace!("alloc(0x{placement:08X}, {size})");

		if next_placement < MAX_ADDR {
			PLACEMENT_ADDR = next_placement;
			placement
		} else {
			0
		}
	};

	ptr
}

pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		kmalloc(layout.size(), layout.align() as u32) as *mut u8
	}

	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		trace!("leak(0x{:08X})", ptr as usize);
	}
}

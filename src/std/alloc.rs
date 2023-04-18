use core::{
	alloc::{GlobalAlloc, Layout},
	fmt::Write,
};

static mut PLACEMENT_ADDR: u32 = 0x200000;
static mut MAX_ADDR: u32 = 0x400000;

const PAGE_SIZE: u32 = 0x1000;

pub fn kmalloc_aligned(size: usize) -> u32 {
	align(PAGE_SIZE);
	kmalloc(size)
}

pub fn kmalloc(size: usize) -> u32 {
	align(usize::BITS);
	unsafe {
		let placement = PLACEMENT_ADDR;
		let next_placement = placement + size as u32;
		kprintf!("alloc(0x{placement:0X}, {size})");

		if next_placement < MAX_ADDR {
			PLACEMENT_ADDR = next_placement;
			placement
		} else {
			0
		}
	}
}

fn align(alignment: u32) {
	unsafe { PLACEMENT_ADDR = (PLACEMENT_ADDR & !(alignment - 1)) + alignment };
}

pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		kmalloc(layout.size()) as *mut u8
	}

	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		kprintf!("leak(0x{:08X})", ptr as usize);
	}
}

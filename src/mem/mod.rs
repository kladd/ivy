use core::arch::asm;

use log::debug;

use crate::{arch::amd64::vmem::PML4, mem::page::Page};

pub mod frame;
pub mod page;

// TODO: Define once (already defined in linker and boot.asm).
pub const KERNEL_VMA: usize = 0xFFFFFF8000000000;
pub const KERNEL_LMA: usize = 0x0000000000100000;
pub const PAGE_SIZE: usize = 0x200000;

#[derive(Debug)]
pub struct VirtualAddress(usize);

#[derive(Debug, Copy, Clone)]
pub struct PhysicalAddress(pub usize);

impl PhysicalAddress {
	pub fn to_virtual<T>(&self) -> *mut T {
		self.to_virtual_addr() as *mut T
	}

	pub fn to_virtual_addr(&self) -> usize {
		self.0 | KERNEL_VMA
	}

	pub fn offset(&self, size: usize) -> PhysicalAddress {
		PhysicalAddress(self.0 + size)
	}
}

impl<T> From<*mut T> for PhysicalAddress {
	fn from(value: *mut T) -> Self {
		PhysicalAddress(value as usize - KERNEL_VMA)
	}
}

// TODO: Sort this out.
pub fn kernel_map(pml4: &mut PML4, start: PhysicalAddress, pages: usize) {
	assert!(pages <= 2, "TODO: ID map more than two pages {}", pages);
	// HACK: Don't touch the first meg.
	let region_start = Page::new(start.clone(), 0).entry();
	if region_start < 0x100000 {
		return;
	}
	debug!(
		"Mapping region starting at 0x{:016X}",
		Page::new(start.clone(), 0).entry()
	);
	let pdp = pml4.get_or_alloc(pml4.index(start.to_virtual_addr()), 0x3);
	let pd = pdp.get_or_alloc(pdp.index(start.to_virtual_addr()), 0x3);
	for i in 0..pages {
		let virt = start.to_virtual_addr() + (i * PAGE_SIZE);
		pd.set(
			pd.index(start.to_virtual_addr() + (i * PAGE_SIZE)),
			Page::new(start.offset(i * PAGE_SIZE), 0x83),
		);
		unsafe { asm!("invlpg [{}]", in(reg) virt) };
	}
}

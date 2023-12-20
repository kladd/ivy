use crate::arch::amd64::vmem::{PageTable, PML4};

pub mod frame;

// TODO: Define once (already defined in linker and boot.asm).
pub const KERNEL_VMA: usize = 0xFFFFFF8000000000;
pub const KERNEL_LMA: usize = 0x0000000000100000;
pub const PAGE_SIZE: usize = 0x200000;

#[derive(Debug)]
pub struct VirtualAddress(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct PhysicalAddress(pub usize);

impl PhysicalAddress {
	pub fn to_virtual<T>(&self) -> *mut T {
		self.to_virtual_addr() as *mut T
	}

	// TODO: This works iff this page is identity (in high half) mapped.
	pub fn to_virtual_addr(&self) -> usize {
		self.0 | KERNEL_VMA
	}

	pub fn offset(&self, size: usize) -> Self {
		PhysicalAddress(self.0 + size)
	}

	pub fn page_align_floor(&self) -> Self {
		PhysicalAddress(self.0 & !(PAGE_SIZE - 1))
	}
}

impl<T> From<*mut T> for PhysicalAddress {
	fn from(value: *mut T) -> Self {
		PhysicalAddress(value as usize - KERNEL_VMA)
	}
}

// TODO: Sort this out.
pub fn kernel_map(
	pml4: &mut PageTable<PML4>,
	start: PhysicalAddress,
	pages: usize,
) {
	let start = start.page_align_floor();

	// HACK: Don't touch the first meg. TODO: I can't remember why.
	if start.0 < 0x100000 {
		return;
	}

	for i in 0..pages {
		pml4.map(
			start.offset(i * PAGE_SIZE).0,
			start.offset(i * PAGE_SIZE).to_virtual_addr(),
			0x83,
			true,
		);
	}
}

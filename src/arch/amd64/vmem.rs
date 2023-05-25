use core::{
	ops::{Index, IndexMut},
	sync::atomic::AtomicPtr,
};

pub struct VirtAddr(usize);
pub struct PhysAddr(usize);

#[repr(align(0x1000))]
pub struct PageTable(pub [*mut PageTable; 512]);

pub static BOOT_PML4_TABLE: AtomicPtr<PageTable> =
	AtomicPtr::new(boot_pml4 as *mut PageTable);
pub static BOOT_PDP_TABLE: AtomicPtr<PageTable> =
	AtomicPtr::new(boot_pdp as *mut PageTable);
pub static BOOT_PD_TABLE: AtomicPtr<PageTable> =
	AtomicPtr::new(boot_pd as *mut PageTable);

impl Index<usize> for PageTable {
	type Output = *mut PageTable;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl IndexMut<usize> for PageTable {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

extern "C" {
	fn boot_pml4();
	fn boot_pdp();
	fn boot_pd();
}

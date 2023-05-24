use core::{
	ops::{Index, IndexMut},
	sync::atomic::AtomicPtr,
};

pub struct VirtAddr(usize);
pub struct PhysAddr(usize);

#[repr(align(0x1000))]
pub struct PageTable(pub [*mut PageTable; 512]);

pub static BOOT_PML4_TABLE: AtomicPtr<PageTable> =
	AtomicPtr::new(pml4_table as *mut PageTable);
pub static BOOT_PDP_TABLE: AtomicPtr<PageTable> =
	AtomicPtr::new(pdp_table as *mut PageTable);
pub static BOOT_PD_TABLE: AtomicPtr<PageTable> =
	AtomicPtr::new(pd_table as *mut PageTable);

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
	fn pml4_table();
	fn pdp_table();
	fn pd_table();
}

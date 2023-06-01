use core::{
	ops::{Index, IndexMut},
	sync::atomic::AtomicPtr,
};

use crate::mem::page::Page;

#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageTable(pub [usize; 512]);

pub const BOOT_PML4_TABLE: *mut PageTable = boot_pml4 as *mut PageTable;
// pub static BOOT_PDP_TABLE: AtomicPtr<PageTable> =
// 	AtomicPtr::new(boot_pdp as *mut PageTable);
// pub static BOOT_PD_TABLE: AtomicPtr<PageTable> =
// 	AtomicPtr::new(boot_pd as *mut PageTable);

impl PageTable {
	pub fn index(addr: usize) -> (usize, usize, usize) {
		let pml4 = ((addr >> 21 >> 9 >> 9) & 0x1FF);
		let pdp = ((addr >> 21 >> 9) & 0x1FF);
		let pd = ((addr >> 21) & 0x1FF);

		(pml4, pdp, pd)
	}
}

impl Index<usize> for PageTable {
	type Output = usize;

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
	// fn boot_pdp();
	// fn boot_pd();
}

use alloc::{alloc::alloc_zeroed, boxed::Box};
use core::{
	alloc::Layout,
	ops::{Index, IndexMut},
};

use crate::mem::{page::Page, PhysicalAddress};

#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageTable(pub [usize; 512]);

pub const BOOT_PML4_TABLE: *mut PageTable = boot_pml4 as *mut PageTable;
pub const KERN_PML4_TABLE: Option<*mut PML4> = Some(boot_pml4 as *mut PML4);
pub const BOOT_PDP_TABLE: *mut PageTable = boot_pdp as *mut PageTable;
pub const BOOT_PD_TABLE: *mut PageTable = boot_pd as *mut PageTable;

#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PML4([usize; 512]);

#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PDP(pub [usize; 512]);

#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PD(pub [usize; 512]);

impl PML4 {
	const MASK: usize = !0xFF;

	pub fn index(&self, addr: usize) -> usize {
		(addr >> 21 >> 9 >> 9) & 0x1FF
	}

	pub fn set(&mut self, index: usize, pdp: Box<PDP>, flags: usize) {
		self.0[index] = PhysicalAddress::from(Box::into_raw(pdp)).0 + flags;
	}

	pub fn get(&self, index: usize) -> Option<&mut PDP> {
		if self.0[index] == 0 {
			None
		} else {
			unsafe {
				Some(
					&mut *(PhysicalAddress(self.0[index] & Self::MASK)
						.to_virtual()),
				)
			}
		}
	}

	pub fn get_or_alloc(&mut self, index: usize, flags: usize) -> &mut PDP {
		if self.0[index] == 0 {
			self.set(index, PDP::alloc(), flags);
		}
		self.get(index).unwrap()
	}

	pub fn alloc() -> Box<Self> {
		unsafe { Box::from_raw(Self::alloc_raw()) }
	}

	fn alloc_raw() -> *mut Self {
		unsafe { alloc_zeroed(Layout::new::<Self>()) as *mut _ }
	}

	pub fn adopt_boot_table() -> Option<&'static mut Self> {
		KERN_PML4_TABLE.map(|s| unsafe { &mut *s }).take()
	}
}

impl PDP {
	const MASK: usize = !0xFF;

	pub fn index(&self, addr: usize) -> usize {
		(addr >> 21 >> 9) & 0x1FF
	}

	pub fn set(&mut self, index: usize, pd: Box<PD>, flags: usize) {
		self.0[index] = PhysicalAddress::from(Box::into_raw(pd)).0 + flags;
	}

	pub fn get(&mut self, index: usize) -> Option<&mut PD> {
		if self.0[index] == 0 {
			None
		} else {
			unsafe {
				Some(
					&mut *(PhysicalAddress(self.0[index] & Self::MASK)
						.to_virtual()),
				)
			}
		}
	}

	pub fn get_or_alloc(&mut self, index: usize, flags: usize) -> &mut PD {
		if self.0[index] == 0 {
			self.set(index, PD::alloc(), flags);
		}
		self.get(index).unwrap()
	}

	pub fn alloc() -> Box<Self> {
		unsafe { Box::from_raw(Self::alloc_raw()) }
	}

	pub fn alloc_raw() -> *mut Self {
		unsafe { alloc_zeroed(Layout::new::<Self>()) as *mut _ }
	}
}

impl PD {
	const MASK: usize = !0xFF;

	pub fn index(&self, addr: usize) -> usize {
		(addr >> 21) & 0x1FF
	}

	pub fn set(&mut self, index: usize, pde: Page) {
		self.0[index] = pde.entry();
	}

	pub fn get(&mut self, index: usize) -> Option<Page> {
		Page::from_entry(self.0[index])
	}

	pub fn alloc() -> Box<Self> {
		unsafe { Box::from_raw(Self::alloc_raw()) }
	}

	pub fn alloc_raw() -> *mut Self {
		unsafe { alloc_zeroed(Layout::new::<Self>()) as *mut _ }
	}
}

impl PageTable {
	pub fn index(addr: usize) -> (usize, usize, usize) {
		let pml4 = ((addr >> 21 >> 9 >> 9) & 0x1FF);
		let pdp = ((addr >> 21 >> 9) & 0x1FF);
		let pd = ((addr >> 21) & 0x1FF);

		(pml4, pdp, pd)
	}

	pub fn alloc() -> Box<Self> {
		unsafe { Box::from_raw(Self::alloc_raw()) }
	}

	pub fn alloc_raw() -> *mut PageTable {
		unsafe { alloc_zeroed(Layout::new::<PageTable>()) as *mut PageTable }
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
	fn boot_pdp();
	fn boot_pd();
}

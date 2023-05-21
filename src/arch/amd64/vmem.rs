use core::{
	arch::asm,
	ops::{Index, IndexMut},
	ptr,
};

const PRESENT: u64 = 0x1;
const WRITE: u64 = 0x2;
const USER: u64 = 0x4;

#[repr(align(4096))]
pub struct PageTableEntry([u64; 512]);

#[repr(transparent)]
pub struct PageTable([PageTableEntry; 512]);

impl Index<usize> for PageTable {
	type Output = PageTableEntry;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl IndexMut<usize> for PageTable {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

impl Index<usize> for PageTableEntry {
	type Output = u64;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index]
	}
}

impl IndexMut<usize> for PageTableEntry {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0[index]
	}
}

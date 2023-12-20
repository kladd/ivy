use alloc::alloc::alloc_zeroed;
use core::{
	alloc::Layout,
	arch::asm,
	marker::PhantomData,
	ops::{Index, IndexMut},
};

use log::{debug, warn};

use crate::mem::{PhysicalAddress, KERNEL_VMA, PAGE_SIZE};

pub trait Table {
	fn index_of(addr: usize) -> usize;
	fn name() -> &'static str;
}
pub trait PageTableDirectory: Table {
	type NextTable: Table;
}

pub enum PML4 {}

impl Table for PML4 {
	fn index_of(addr: usize) -> usize {
		(addr >> 21 >> 9 >> 9) & 0x1FF
	}

	fn name() -> &'static str {
		"PML4"
	}
}

impl PageTableDirectory for PML4 {
	type NextTable = PDP;
}

pub enum PDP {}

impl Table for PDP {
	fn index_of(addr: usize) -> usize {
		(addr >> 21 >> 9) & 0x1FF
	}

	fn name() -> &'static str {
		"PDP"
	}
}

impl PageTableDirectory for PDP {
	type NextTable = PD;
}

pub enum PD {}

impl Table for PD {
	fn index_of(addr: usize) -> usize {
		(addr >> 21) & 0x1FF
	}

	fn name() -> &'static str {
		"PD"
	}
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Page(u64);

impl Page {
	pub const PRESENT: u64 = 0x1 << 0;
	pub const READ_WRITE: u64 = 0x1 << 1;
	pub const USER: u64 = 0x1 << 2;
	pub const HUGE: u64 = 0x1 << 7;

	pub fn new(addr: PhysicalAddress, flags: u64) -> Self {
		assert_eq!(addr.0 % 0x1000, 0);
		let pg = Self(addr.0 as u64 | flags);
		pg
	}

	pub fn has(&self, flags: u64) -> bool {
		(self.0 & flags) == flags
	}

	pub fn set(&mut self, flags: u64) {
		self.0 &= flags;
	}

	pub fn unset(&mut self, flags: u64) {
		self.0 &= !flags;
	}

	pub fn address(&self) -> PhysicalAddress {
		PhysicalAddress((self.0 & !0x1FF) as usize)
	}
}

#[repr(align(0x1000))]
#[repr(C)]
pub struct PageTable<L: Table> {
	entries: [Page; 512],
	level: PhantomData<L>,
}

impl PageTable<PML4> {
	pub fn map(&mut self, phys: usize, virt: usize, flags: u64, invlpg: bool) {
		assert_eq!(phys % PAGE_SIZE, 0);
		assert_eq!(virt % PAGE_SIZE, 0);

		// TODO: All pages are huge.
		let pd = self
			.next_alloc(PML4::index_of(virt), flags & !Page::HUGE)
			.next_alloc(PDP::index_of(virt), flags & !Page::HUGE);

		if pd.entries[PD::index_of(virt)].has(Page::PRESENT) {
			warn!("Not mapping present page P: {phys:016X?}, V: {virt:016X?}");
			return;
		}

		pd.entries[PD::index_of(virt)] =
			Page::new(PhysicalAddress(phys), flags | Page::HUGE);

		if invlpg {
			unsafe { asm!("invlpg [{}]", in(reg) virt) };
		}
	}

	pub fn current_mut() -> &'static mut PageTable<PML4> {
		unsafe {
			let mut cr3: usize;
			asm!( "mov {}, cr3", out(reg) cr3 );
			&mut *PhysicalAddress(cr3).to_virtual()
		}
	}

	pub fn new_with_kernel() -> &'static mut Self {
		let current = Self::current_mut();
		let new = unsafe { &mut *(Self::alloc() as *mut Self) };

		for i in 0..current.entries.len() {
			if current.entries[i].has(Page::PRESENT)
				&& !current.entries[i].has(Page::USER)
			{
				new.entries[i] = current.entries[i];
			}
		}

		new
	}
}

impl<L: Table> PageTable<L> {
	fn index_of(&self, addr: usize) -> usize {
		L::index_of(addr)
	}

	fn alloc() -> usize {
		unsafe { alloc_zeroed(Layout::new::<Self>()) as *mut _ as usize }
	}
}

impl<L: Table> Index<usize> for PageTable<L> {
	type Output = Page;

	fn index(&self, index: usize) -> &Self::Output {
		&self.entries[index]
	}
}

impl<L: Table> IndexMut<usize> for PageTable<L> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.entries[index]
	}
}

impl<L: PageTableDirectory> PageTable<L> {
	pub fn next(&mut self, idx: usize) -> Option<&mut PageTable<L::NextTable>> {
		if self.entries[idx].has(Page::PRESENT) {
			Some(unsafe { &mut *self.entries[idx].address().to_virtual() })
		} else {
			None
		}
	}

	pub fn next_alloc(
		&mut self,
		idx: usize,
		flags: u64,
	) -> &mut PageTable<L::NextTable> {
		if self.entries[idx].has(Page::PRESENT) {
			return self.next(idx).unwrap();
		}

		debug!(
			"{}[{}] is missing, allocating ({flags:02X})",
			L::name(),
			idx
		);

		let addr = Self::alloc();
		self.entries[idx] =
			Page::new(PhysicalAddress(addr - KERNEL_VMA), flags & !Page::HUGE);

		self.next(idx).unwrap()
	}
}

pub fn page_table_index(vaddr: usize) -> (usize, usize, usize) {
	(
		PML4::index_of(vaddr),
		PDP::index_of(vaddr),
		PD::index_of(vaddr),
	)
}

pub fn map_physical_memory(size: usize) {
	assert_eq!(size % PAGE_SIZE, 0);

	let pml4 = PageTable::<PML4>::current_mut();

	for addr in (0..size).step_by(PAGE_SIZE) {
		pml4.map(
			addr,
			PhysicalAddress(addr).to_virtual_addr(),
			Page::PRESENT | Page::READ_WRITE,
			true,
		);
	}
}

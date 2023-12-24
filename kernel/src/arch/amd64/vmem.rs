use alloc::alloc::alloc_zeroed;
use core::{
	alloc::Layout,
	arch::asm,
	marker::PhantomData,
	ops::{Index, IndexMut},
	ptr,
};

use log::{debug, warn};

use crate::mem::{frame, PhysicalAddress, KERNEL_VMA, PAGE_SIZE};

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

	pub fn flags(&self) -> u64 {
		self.0 & 0x1FF
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
			let entry = current.entries[i];
			if entry.has(Page::PRESENT) && !entry.has(Page::USER) {
				new.entries[i] = entry;
			}
		}

		new
	}

	pub fn fork(&self) -> &'static mut Self {
		let new = Self::new_with_kernel();
		copy_user_page_directories(self, new);
		new
	}
}

// TODO: Needs more nesting.
pub fn debug_page_directory(dir: &PageTable<PML4>, filter: u64) {
	for i in 0..dir.entries.len() {
		let entry = dir.entries[i];
		if entry.has(filter) {
			debug!("[{i:03}] {:016X}:{:03X}", entry.address().0, entry.flags());
			let Some(pdp) = dir.next(i) else {
				continue;
			};

			for i in 0..pdp.entries.len() {
				let entry = pdp.entries[i];
				if entry.has(filter) {
					debug!(
						"    [{i:03}] {:016X}:{:03X}",
						entry.address().0,
						entry.flags()
					);
					let Some(pd) = pdp.next(i) else {
						continue;
					};

					for i in 0..pd.entries.len() {
						let entry = pd.entries[i];
						if entry.has(filter) {
							debug!(
								"        [{i:03}] {:016X}:{:03X}",
								entry.address().0,
								entry.flags()
							);
						}
					}
				}
			}
		}
	}
}

// TODO: Needs more nesting.
fn copy_user_page_directories(
	src: &PageTable<PML4>,
	dst: &mut PageTable<PML4>,
) {
	for i in 0..src.entries.len() {
		if src.entries[i].has(Page::PRESENT | Page::USER) {
			let pdp_new = dst.next_alloc(i, 7);
			let pdp_cur = src.next(i).unwrap();

			for i in 0..pdp_cur.entries.len() {
				if pdp_cur.entries[i].has(Page::PRESENT | Page::USER) {
					let pd_new = pdp_new.next_alloc(i, 7);
					let pd_cur = pdp_cur.next(i).unwrap();

					copy_user_page_table(pd_cur, pd_new);
				}
			}
		}
	}
}

fn copy_user_page_table(src: &PageTable<PD>, dst: &mut PageTable<PD>) {
	let mut falloc = frame::current_mut().lock();
	for i in 0..src.entries.len() {
		if src.entries[i].has(Page::PRESENT | Page::USER) {
			dst[i] = Page::new(falloc.alloc(), 0x87);
			unsafe {
				ptr::copy_nonoverlapping(
					src.entries[i].address().to_virtual() as *const u8,
					dst[i].address().to_virtual(),
					PAGE_SIZE,
				);
			}
		}
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
	pub fn next(&self, idx: usize) -> Option<&PageTable<L::NextTable>> {
		if self.entries[idx].has(Page::PRESENT) {
			Some(unsafe { &mut *self.entries[idx].address().to_virtual() })
		} else {
			None
		}
	}

	pub fn next_mut(
		&mut self,
		idx: usize,
	) -> Option<&mut PageTable<L::NextTable>> {
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
			return self.next_mut(idx).unwrap();
		}

		debug!(
			"{}[{}] is missing, allocating ({flags:02X})",
			L::name(),
			idx
		);

		let addr = Self::alloc();
		self.entries[idx] =
			Page::new(PhysicalAddress(addr - KERNEL_VMA), flags & !Page::HUGE);

		self.next_mut(idx).unwrap()
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

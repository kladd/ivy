use alloc::{
	alloc::{alloc_zeroed, Global},
	borrow::ToOwned,
	boxed::Box,
	vec,
	vec::Vec,
};
use core::{
	alloc::Layout,
	arch::asm,
	sync::atomic::{AtomicU64, Ordering},
};

use log::{debug, warn};

use crate::{
	arch::amd64::vmem::{PageTable, BOOT_PML4_TABLE},
	mem::{frame::FrameAllocator, page::Page, KERNEL_BASE, PAGE_SIZE},
};

static NEXT_PID: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
pub struct CPU {
	pub rsp0: usize,
	pub rsp3: usize,
}

impl CPU {
	pub fn store(&mut self) {
		unsafe { asm!("wrgsbase {}", in(reg) self as *mut Self) };
	}
}

#[derive(Debug)]
pub struct Task {
	pid: u64,
	name: &'static str,
	pub rbp: usize,
	pub rsp: usize,
	pub rip: usize,
	pub cr3: usize,
}

impl Task {
	const STACK_SIZE: usize = 0x1000;
	const STACK_ALIGN: usize = 0x1000;
	const START_ADDR: usize = 0x400000;

	pub fn new(name: &'static str) -> Self {
		let rbp = Self::START_ADDR + PAGE_SIZE - Self::STACK_SIZE;
		let rsp = Self::START_ADDR + PAGE_SIZE;

		let frame = FrameAllocator::alloc();
		let page = Page::new(frame, 0x7);

		// Start at 4MB for no particular reason.
		let (pml4_i, pdp_i, pd_i) = PageTable::index(Self::START_ADDR);

		// Copy the existing PML4 table, which maps the kernel already.
		let mut pml4 = unsafe {
			let table =
				alloc_zeroed(Layout::new::<PageTable>()) as *mut PageTable;
			table.copy_from(BOOT_PML4_TABLE, 1);
			debug!("TABLE: 0x{table:016X?}");
			Box::from_raw(table)
		};
		let mut pdp = unsafe {
			Box::from_raw(
				alloc_zeroed(Layout::new::<PageTable>()) as *mut PageTable
			)
		};
		let mut pd = unsafe {
			Box::from_raw(
				alloc_zeroed(Layout::new::<PageTable>()) as *mut PageTable
			)
		};
		// debug!("page.entry");
		pd[pd_i] = page.entry();
		for (i, ent) in pd.0.iter().enumerate() {
			if *ent != 0 {
				warn!("[{i}] = {ent:016X}");
			}
		}
		// TODO: V2P. This is brittle. Also, you know, plan the virtual address.
		// debug!("box leak 1");
		pdp[pdp_i] = Box::leak(pd) as *mut _ as usize - KERNEL_BASE + 7;
		for (i, ent) in pdp.0.iter().enumerate() {
			if *ent != 0 {
				warn!("[{i}] = {ent:016X}");
			}
		}
		// debug!("box leak 2");
		pml4[pml4_i] = Box::leak(pdp) as *mut _ as usize - KERNEL_BASE + 7;
		for (i, ent) in pml4.0.iter().enumerate() {
			if *ent != 0 {
				warn!("[{i}] = {ent:016X}");
			}
		}

		let cr3 = Box::leak(pml4) as *mut _ as usize;

		Self {
			pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
			name,
			rbp,
			rsp,
			rip: Self::START_ADDR,
			cr3: cr3 - KERNEL_BASE, // to physical
		}
	}
}

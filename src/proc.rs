use alloc::{alloc::alloc_zeroed, boxed::Box};
use core::{
	alloc::Layout,
	arch::asm,
	intrinsics::size_of,
	sync::atomic::{AtomicU64, Ordering},
};

use log::{debug, warn};

use crate::{
	arch::amd64::{
		idt::InterruptEntry,
		vmem::{PageTable, BOOT_PML4_TABLE},
	},
	mem::{frame::FrameAllocator, page::Page, KERNEL_VMA, PAGE_SIZE},
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
	pub const START_ADDR: usize = 0x1000;

	pub fn new(falloc: &mut FrameAllocator, name: &'static str) -> Self {
		let rbp = Self::START_ADDR + PAGE_SIZE - Self::STACK_SIZE;
		let rsp = Self::START_ADDR + PAGE_SIZE - size_of::<InterruptEntry>();

		let frame = falloc.alloc();
		let page = Page::new(frame, 0x87);

		debug!("page entry {:016X}", page.entry());

		let (pml4_i, pdp_i, pd_i) = PageTable::index(Self::START_ADDR);

		// Copy the existing PML4 table, which maps the kernel already.
		let mut pml4 = unsafe {
			let table = kdbg!(alloc_zeroed(Layout::new::<PageTable>()))
				as *mut PageTable;
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
		pd[pd_i] = page.entry();
		for (i, ent) in pd.0.iter().enumerate() {
			if *ent != 0 {
				warn!("pd[{i}] = {ent:016X}");
			}
		}
		// TODO: Flags.
		pdp[pdp_i] = Box::leak(pd) as *mut _ as usize - KERNEL_VMA + 0x7;
		for (i, ent) in pdp.0.iter().enumerate() {
			if *ent != 0 {
				warn!("pdp[{i}] = {ent:016X}");
			}
		}
		pml4[pml4_i] = Box::leak(pdp) as *mut _ as usize - KERNEL_VMA + 0x7;
		for (i, ent) in pml4.0.iter().enumerate() {
			if *ent != 0 {
				warn!("pml4[{i}] = {ent:016X}");
			}
		}

		let cr3 = Box::leak(pml4) as *mut _ as usize;

		Self {
			pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
			name,
			rbp,
			rsp,
			rip: Self::START_ADDR,
			cr3: cr3 - KERNEL_VMA, // to physical
		}
	}
}

use alloc::{alloc::alloc_zeroed, boxed::Box, vec::Vec};
use core::{
	alloc::Layout,
	arch::asm,
	intrinsics::size_of,
	ptr,
	sync::atomic::{AtomicU64, Ordering},
};

use log::debug;

use crate::{
	arch::amd64::{
		idt::InterruptEntry,
		vmem::{PageTable, BOOT_PML4_TABLE},
	},
	fs,
	fs::{device::inode::DeviceInode, fs0, inode::Inode, FileDescriptor},
	mem::{frame, page::Page, KERNEL_VMA, PAGE_SIZE},
};

static NEXT_PID: AtomicU64 = AtomicU64::new(0);

pub struct CPU {
	pub rsp0: usize,
	pub rsp3: usize,
	pub task: *mut Task,
}

impl CPU {
	pub fn store(&mut self) {
		unsafe { asm!("wrgsbase {}", in(reg) self as *mut Self) };
	}

	pub fn load() -> &'static Self {
		let mut cpu: u64;
		unsafe {
			asm!("rdgsbase {}", out(reg) cpu);
			&*(cpu as *mut Self)
		}
	}

	pub fn new() -> Self {
		Self {
			rsp0: 0,
			rsp3: 0,
			task: ptr::null_mut(),
		}
	}
}

#[derive(Debug)]
pub struct Task {
	pid: u64,
	name: &'static str,
	pub open_files: Vec<fs::FileDescriptor>,
	pub cwd: Inode,
	pub rbp: usize,
	pub rsp: usize,
	pub rip: usize,
	pub cr3: usize,
}

impl Task {
	const STACK_SIZE: usize = 0x1000;
	const STACK_ALIGN: usize = 0x1000;
	pub const START_ADDR: usize = 0x200000;

	pub fn new(name: &'static str) -> Self {
		let rbp = PAGE_SIZE;
		let rsp = 2 * PAGE_SIZE - size_of::<InterruptEntry>();

		let mut falloc = frame::current_mut().lock();

		// HACK: Programs better not exceed 2MB!!
		let program_page = Page::new(falloc.alloc(), 0x87);
		let stack_page = Page::new(falloc.alloc(), 0x87);

		debug!("page entry {:016X}", program_page.entry());
		debug!("stack entry {:016X}", stack_page.entry());

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
		pd[pd_i] = program_page.entry();
		// HACK: Stack is code page plus one..
		pd[pd_i + 1] = stack_page.entry();
		// TODO: Flags.
		pdp[pdp_i] = Box::leak(pd) as *mut _ as usize - KERNEL_VMA + 0x7;
		pml4[pml4_i] = Box::leak(pdp) as *mut _ as usize - KERNEL_VMA + 0x7;

		let cr3 = Box::leak(pml4) as *mut _ as usize;

		let mut open_files = Vec::with_capacity(3);
		open_files
			.push(FileDescriptor::new(Inode::Device(DeviceInode::Console)));
		open_files
			.push(FileDescriptor::new(Inode::Device(DeviceInode::Console)));
		open_files
			.push(FileDescriptor::new(Inode::Device(DeviceInode::Serial)));

		Self {
			pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
			cwd: fs0().find(&fs0().root(), "/home/default").unwrap(),
			open_files,
			name,
			rbp,
			rsp,
			rip: Self::START_ADDR,
			cr3: cr3 - KERNEL_VMA, // to physical
		}
	}
}

impl Drop for Task {
	fn drop(&mut self) {
		panic!("you dropped a task and didn't mean to");
	}
}

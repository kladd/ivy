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
	arch::amd64::{idt::InterruptEntry, vmem, vmem::PML4},
	fs,
	fs::{device::inode::DeviceInode, fs0, inode::Inode, FileDescriptor},
	mem::{frame, KERNEL_VMA, PAGE_SIZE},
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

	pub fn current_task(&self) -> &mut Task {
		unsafe { &mut *self.task }
	}
}

#[derive(Debug)]
pub struct Task {
	pub pid: u64,
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
	pub const STACK_BOTTOM: usize = 0xFFFFFE8000000000;

	pub fn new(name: &'static str) -> Self {
		let mut falloc = frame::current_mut().lock();

		// Copy the existing PML4 table, which maps the kernel already.
		let pml4 = vmem::PageTable::<PML4>::new_with_kernel();
		let cr3 = pml4 as *mut _ as usize;

		// HACK: Programs better not exceed 2MB!!
		let rbp = Self::STACK_BOTTOM;
		let rsp = Self::STACK_BOTTOM + PAGE_SIZE - 16;
		pml4.map(
			falloc.alloc().0,
			Self::START_ADDR,
			vmem::Page::HUGE
				| vmem::Page::PRESENT
				| vmem::Page::USER
				| vmem::Page::READ_WRITE,
			false,
		);
		pml4.map(
			falloc.alloc().0,
			Self::STACK_BOTTOM,
			vmem::Page::PRESENT | vmem::Page::USER | vmem::Page::READ_WRITE,
			false,
		);

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

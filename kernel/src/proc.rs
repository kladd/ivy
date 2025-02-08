use alloc::{alloc::alloc, boxed::Box, vec::Vec};
use core::{
	alloc::Layout,
	arch::asm,
	mem::ManuallyDrop,
	ptr,
	sync::atomic::{AtomicU64, Ordering},
};

use log::{debug, info, trace};
use vmem::{Page, PageTable};

use crate::{
	arch::amd64::{cli, vmem, vmem::PML4},
	fs,
	fs::{device::inode::DeviceInode, fs0, inode::Inode, FileDescriptor},
	mem::{frame, PhysicalAddress, KERNEL_VMA, PAGE_SIZE},
	syscall::RegisterState,
};

static NEXT_PID: AtomicU64 = AtomicU64::new(1);

pub struct CPU {
	pub rsp0: usize,
	pub rsp3: usize,
	pub task: *mut Task,
}

impl CPU {
	pub fn store(&mut self) {
		unsafe { asm!("wrgsbase {}", in(reg) self as *mut Self) };
	}

	pub fn load() -> &'static mut Self {
		let mut cpu: u64;
		unsafe {
			asm!("rdgsbase {}", out(reg) cpu);
			&mut *(cpu as *mut Self)
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

	pub fn switch_task(&mut self, next_task: &mut Task) {
		// TODO: Push/pop interrupt flag, though sysret will enable interrupts
		//       anyway.
		cli();
		info!("new task: {next_task:#X?}");
		// Store current task.
		self.task = next_task as *mut Task;
		// Store user-space stack pointer.
		self.rsp3 = next_task.register_state.rsp as usize;
		// Switch page tables.
		unsafe { asm!("mov cr3, {}", in(reg) next_task.cr3) };
	}
}

#[derive(Debug)]
pub struct Task {
	name: &'static str,
	pub pid: u64,
	pub cwd: Inode,
	pub open_files: Vec<fs::FileDescriptor>,

	pub cr3: usize,
	pub register_state: RegisterState,

	pub next: *mut Task,
}

impl Task {
	const STACK_SIZE: usize = 0x1000;
	const STACK_ALIGN: usize = 0x1000;
	pub const START_ADDR: usize = 0x200000;
	pub const STACK_BOTTOM: usize = 0xFFFFFE8000000000;

	pub fn new(name: &'static str) -> Self {
		let mut open_files = Vec::with_capacity(3);
		open_files
			.push(FileDescriptor::new(Inode::Device(DeviceInode::Console)));
		open_files
			.push(FileDescriptor::new(Inode::Device(DeviceInode::Console)));
		open_files
			.push(FileDescriptor::new(Inode::Device(DeviceInode::Serial)));

		let mut fetus = Self {
			pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
			cwd: fs0().find(&fs0().root(), "/home/default").unwrap(),
			open_files,
			name,
			register_state: RegisterState::default(),
			cr3: 0,
			next: ptr::null_mut(),
		};
		fetus.reimage();
		fetus
	}

	pub fn reimage(&mut self) {
		let mut falloc = frame::current_mut().lock();

		// Copy the existing PML4 table, which maps the kernel already.
		let pml4 = PageTable::<PML4>::new_with_kernel();
		let cr3 = pml4 as *mut _ as usize;

		// HACK: Programs better not exceed 2MB!!
		let rbp = Self::STACK_BOTTOM;
		let rsp = Self::STACK_BOTTOM + PAGE_SIZE - 16;
		pml4.map(
			falloc.alloc().0,
			Self::START_ADDR,
			Page::HUGE | Page::PRESENT | Page::USER | Page::READ_WRITE,
			false,
		);
		pml4.map(
			falloc.alloc().0,
			Self::STACK_BOTTOM,
			Page::PRESENT | Page::USER | Page::READ_WRITE,
			false,
		);

		self.register_state.rsp = rsp as u64;
		self.register_state.rbp = rbp as u64;
		self.register_state.rip = Self::START_ADDR as u64;
		self.cr3 = cr3 - KERNEL_VMA;
	}

	pub fn fork(&mut self) -> &'static mut Task {
		trace!("Task::fork()");

		let pml4: &mut PageTable<PML4> =
			unsafe { &mut *(PhysicalAddress(self.cr3).to_virtual()) };
		let child_pml4 = pml4.fork();

		let task = Self {
			pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
			cwd: self.cwd.clone(),
			open_files: self.open_files.clone(),
			name: self.name.clone(),
			register_state: self.register_state.clone(),
			cr3: (child_pml4 as *mut PageTable<PML4> as usize) - KERNEL_VMA,
			// to physical
			next: ptr::null_mut(),
		};

		unsafe { &mut *(Box::into_raw(Box::new(task))) }
	}
}

impl Drop for Task {
	fn drop(&mut self) {
		panic!("you dropped a task and didn't mean to");
	}
}

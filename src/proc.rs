use core::{
	mem::size_of,
	ptr,
	sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use log::warn;

use crate::{
	arch::x86::{disable_interrupts, halt},
	fs::{file_descriptor::FileDescriptor, inode::Inode},
	std::alloc::kmalloc_aligned,
};

static PID_COUNTER: AtomicU32 = AtomicU32::new(0);
static CURRENT_TASK: AtomicPtr<Task> = AtomicPtr::new(0 as *mut Task);

#[derive(Debug)]
#[repr(C)]
pub struct Task {
	pid: u32,
	esp: u32,
	cr3: u32,
	pub cwd: Inode,
}

#[repr(packed)]
struct TaskStackFrame {
	ebp: u32,
	ebx: u32,
	esi: u32,
	edi: u32,
	eip: u32,
	ret: u32,
}

impl Task {
	const STACK_SIZE: u32 = 0x1000;

	pub fn new(entry: fn(), cwd: Inode) -> Self {
		let stack_bottom = kmalloc_aligned(Self::STACK_SIZE as usize * 2);

		let eip = entry as u32;
		let ebp = stack_bottom + Self::STACK_SIZE;
		let esp = ebp - size_of::<TaskStackFrame>() as u32;
		let task_stack_frame = TaskStackFrame {
			ebx: 0,
			esi: 0,
			edi: 0,
			ebp,
			eip,
			ret: kernel_idle as u32,
		};

		unsafe { *(esp as *mut TaskStackFrame) = task_stack_frame };

		Task {
			pid: PID_COUNTER.fetch_add(1, Ordering::SeqCst),
			esp,
			// TODO: Allocate a new page directory.
			cr3: 0,
			cwd,
		}
	}

	pub fn set_current(task: &Task) {
		CURRENT_TASK.store(task as *const _ as *mut _, Ordering::Relaxed);
	}

	pub fn current() -> *mut Task {
		CURRENT_TASK.load(Ordering::Relaxed)
	}

	pub fn chdir(&self, fd: &FileDescriptor) {
		unsafe {
			ptr::replace::<Inode>(&self.cwd as *const _ as *mut _, fd.inode)
		};
	}
}

pub fn schedule(task: &Task) -> ! {
	CURRENT_TASK.store(task as *const _ as *mut _, Ordering::Relaxed);
	disable_interrupts();
	unsafe { switch_task(task) };
	unreachable!();
}

extern "C" {
	#[allow(improper_ctypes)]
	fn switch_task(task: &Task) -> u32;
}

pub fn kernel_idle() -> ! {
	warn!("Idle task");
	loop {
		halt();
	}
}

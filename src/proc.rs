use core::{
	fmt::Write,
	mem::size_of,
	ptr,
	sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use crate::{
	arch::x86::halt,
	fat::{DirectoryEntryNode, FATFileSystem},
	fs::File,
	std::alloc::kmalloc_aligned,
	switch_task,
};

static PID_COUNTER: AtomicU32 = AtomicU32::new(0);
static CURRENT_TASK: AtomicPtr<Task> = AtomicPtr::new(0 as *mut Task);

#[repr(C)]
#[derive(Debug)]
pub struct Task<'a> {
	pid: u32,
	esp: u32,
	cr3: u32,
	pub fs: &'a FATFileSystem,
	pub cwd: &'a DirectoryEntryNode,
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

impl<'a> Task<'a> {
	const STACK_SIZE: u32 = 0x1000;

	pub fn new(
		entry: fn(),
		fs: &'a FATFileSystem,
		cwd: &'a DirectoryEntryNode,
	) -> Self {
		let stack_bottom = kmalloc_aligned(Self::STACK_SIZE as usize);

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
			fs,
			cwd,
		}
	}

	pub fn current() -> *mut Task<'static> {
		CURRENT_TASK.load(Ordering::Relaxed)
	}

	pub fn chdir(&self, f: &File) {
		match f {
			File::Directory(f) => unsafe {
				ptr::replace::<DirectoryEntryNode>(
					self.cwd as *const _ as *mut _,
					f.entry(),
				);
			},
			_ => return,
		}
	}
}

pub fn schedule(task: &Task) -> ! {
	CURRENT_TASK.store(task as *const _ as *mut _, Ordering::Relaxed);
	unsafe { switch_task(task) };
	unreachable!();
}

fn kernel_idle() -> ! {
	kprintf!("IDLE");
	loop {
		halt();
	}
}

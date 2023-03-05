use core::{
	mem::size_of,
	sync::atomic::{AtomicU32, Ordering},
};

use crate::std::alloc::kmalloc_aligned;

static PID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
#[derive(Debug)]
pub struct Task {
	pid: u32,
	esp: u32,
	cr3: u32,
}

#[repr(packed)]
struct TaskStackFrame {
	ebx: u32,
	esi: u32,
	edi: u32,
	ebp: u32,
	eip: u32,
}

impl TaskStackFrame {
	fn new(eip: u32, ebp: u32) -> Self {
		Self {
			ebx: 0,
			esi: 0,
			edi: 0,
			ebp,
			eip,
		}
	}
}

impl Task {
	const STACK_SIZE: u32 = 0x1000;

	pub fn new(entry: fn()) -> Self {
		let stack_bottom = kmalloc_aligned(Self::STACK_SIZE as usize);

		let eip = entry as u32;
		let ebp = stack_bottom + Self::STACK_SIZE;
		let esp = ebp - size_of::<TaskStackFrame>() as u32;

		unsafe {
			*(esp as *mut TaskStackFrame) = TaskStackFrame::new(eip, ebp)
		};

		Task {
			pid: PID_COUNTER.fetch_add(1, Ordering::SeqCst),
			esp,
			// TODO: Allocate a new page directory.
			cr3: 0,
		}
	}
}

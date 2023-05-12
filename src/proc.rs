use core::{
	arch::asm,
	ptr,
	ptr::null_mut,
	sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use log::warn;

use crate::{
	arch::x86::{disable_interrupts, halt},
	fs::{file_descriptor::FileDescriptor, inode::Inode},
	std::alloc::kmalloc_aligned,
};

static PID_COUNTER: AtomicU32 = AtomicU32::new(0);
static CURRENT_TASK: AtomicPtr<Task> = AtomicPtr::new(null_mut());

#[derive(Debug)]
pub struct Task {
	pid: u32,
	name: &'static str,
	registers: Registers,
	pub cwd: Inode,
}

#[repr(C)]
#[derive(Debug, Default)]
struct Registers {
	eax: u32,
	ebx: u32,
	ecx: u32,
	edx: u32,
	esi: u32,
	edi: u32,
	esp: u32,
	ebp: u32,
	eip: u32,
	efl: u32,
	cr3: u32,
}

impl Registers {
	const STACK_SIZE: usize = 0x1000;

	// TODO: Page directory.
	pub fn new(entry: fn()) -> Self {
		let mut registers = Self::default();

		unsafe {
			asm!(
				"mov {cr3}, cr3",
				"pushfd; pop {eflags}",
				cr3 = out(reg) registers.cr3,
				eflags = out(reg) registers.efl
			)
		};

		registers.eip = entry as u32;
		registers.ebp = kmalloc_aligned(Self::STACK_SIZE);
		registers.esp = registers.ebp + Self::STACK_SIZE as u32;

		registers
	}
}

impl Task {
	pub fn new(name: &'static str, cwd: Inode, entry: fn()) -> Self {
		Self {
			name,
			pid: PID_COUNTER.fetch_add(1, Ordering::SeqCst),
			cwd,
			registers: Registers::new(entry),
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

pub fn schedule(task: Task) -> ! {
	disable_interrupts();
	Task::set_current(&task);
	unsafe { switch_task(&Registers::new(kernel_idle), &task.registers) };
	unreachable!();
}

extern "C" {
	#[allow(improper_ctypes)]
	fn switch_task(from: &Registers, to: &Registers) -> u32;
}

pub fn kernel_idle() {
	warn!("Idle task");
	loop {
		halt();
	}
}

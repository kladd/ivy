use core::{
	fmt::Write,
	sync::atomic::{AtomicU8, Ordering},
};

use log::{debug, info, trace};

use crate::{
	arch::amd64::{
		clock,
		vmem::{PageTable, PML4},
	},
	devices::{serial, video::vd0},
	mem::{PhysicalAddress, PAGE_SIZE},
	proc::CPU,
};

#[repr(C)]
pub struct RegisterState {
	rcx: u64,
	rdi: u64,
	rax: u64,
}

#[no_mangle]
pub unsafe extern "C" fn syscall_enter(regs: &RegisterState) -> isize {
	trace!("syscall {:03}", regs.rax);
	match regs.rax {
		// TODO: mmap
		12 => return sys_brk(regs.rdi),
		60 => sys_exit(regs.rdi as isize),
		400 => putc(regs.rdi as u8 as char),
		401 => uptime(),
		402 => video(),
		_ => trace!("unknown syscall: {}", regs.rax),
	}
	0
}

fn putc(c: char) {
	writeln!(serial::com1().lock(), "{c}").unwrap();
}

fn uptime() {
	writeln!(serial::com1().lock(), "{}", clock::uptime_seconds()).unwrap();
}

fn video() {
	vd0().test();
}

fn sys_exit(status: isize) {
	info!("program exited with status: {status}");
	breakpoint!();
}

fn sys_brk(addr: u64) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &*cpu.task };

	let pml4: &PML4 = unsafe { &*(PhysicalAddress(task.cr3).to_virtual()) };
	let pdp = pml4.get(0).unwrap();
	let pd = pdp.get(0).unwrap();

	let mut brk = -1;
	let (_, _, start) = PageTable::index(task.rip);
	for i in kdbg!(start)..512 {
		if pd.get(i).is_none() {
			brk = (i * PAGE_SIZE) as isize;
			break;
		}
	}
	trace!("brk: {brk:016X}");

	0
}

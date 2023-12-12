use core::{arch::asm, cmp::min, fmt::Write, ptr, slice, str};

use log::{debug, info, trace};

use crate::{
	arch::amd64::{
		clock,
		vmem::{PageTable, PML4},
	},
	devices::{serial, tty::tty0},
	mem::{frame, page::Page, PhysicalAddress, PAGE_SIZE},
	proc::CPU,
};

#[repr(C)]
pub struct RegisterState {
	rcx: u64,
	rdx: u64,
	rsi: u64,
	rdi: u64,
	rax: u64,
}

#[no_mangle]
pub unsafe extern "C" fn syscall_enter(regs: &RegisterState) -> isize {
	trace!("syscall {:03}", regs.rax);
	match regs.rax {
		// TODO: mmap
		1 => sys_exit(regs.rdi as isize),
		3 => {
			return sys_read(
				regs.rdi as isize,
				regs.rsi as *mut u8,
				regs.rdx as usize,
			) as isize
		}
		4 => sys_write(
			regs.rdi as isize,
			regs.rsi as *const u8,
			regs.rdx as usize,
		),
		69 => return sys_brk(regs.rdi),
		401 => uptime(),
		403 => debug_long(regs.rdi),
		_ => trace!("unknown syscall: {}", regs.rax),
	}
	0
}

fn uptime() {
	writeln!(serial::com1().lock(), "{}", clock::uptime_seconds()).unwrap();
}

fn debug_long(long: u64) {
	debug!("debug_long: {long:016X}");
}

fn sys_read(_fd: isize, ptr: *mut u8, len: usize) -> usize {
	let line = tty0().lock().read_line();

	let mut bytes_written = 0;
	while bytes_written < min(line.len(), len) {
		unsafe {
			*ptr.offset(bytes_written as isize) = line.as_bytes()[bytes_written]
		};
		bytes_written += 1;
	}

	bytes_written
}

fn sys_write(_fd: isize, ptr: *const u8, len: usize) {
	let slice = unsafe { slice::from_raw_parts(ptr, len) };
	let str = str::from_utf8(slice).expect("Invalid UTF-8 string");
	write!(tty0().lock(), "{str}").expect("TTY write error");
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

	let (_, _, start) = PageTable::index(task.rip);

	for i in start..512 {
		if pd.get(i).is_none() {
			let brk = (i * PAGE_SIZE) as isize;

			if addr == 0 {
				trace!("brk: {brk:016X}");
			} else {
				// TODO: Move this whole mess and implement mmap instead.
				let mut frame_allocator = frame::current_mut().lock();
				pd.set(i, Page::new(frame_allocator.alloc(), 0x87));
				unsafe {
					// TODO: This only works if this PD is at PDP[0], PML4[0].
					asm!("invlpg [{}]", in(reg) brk);
				};
			}

			return brk;
		}
	}

	-1
}

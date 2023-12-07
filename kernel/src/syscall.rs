use core::{arch::asm, fmt::Write, ptr, slice, str};

use log::{debug, info, trace};

use crate::{
	arch::amd64::{
		clock,
		vmem::{PageTable, PML4},
	},
	devices::{serial, video::vd0, video_terminal::vdt0},
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
		1 => sys_write(
			regs.rdi as isize,
			regs.rsi as *const u8,
			regs.rdx as usize,
		),
		12 => return sys_brk(regs.rdi),
		60 => sys_exit(regs.rdi as isize),
		401 => uptime(),
		402 => video_test(),
		403 => debug_long(regs.rdi),
		404 => video_clear(),
		405 => return read_line(regs.rdi as *mut u8),
		406 => print_line(regs.rdi as *const u8, regs.rsi as usize),
		_ => trace!("unknown syscall: {}", regs.rax),
	}
	0
}

fn uptime() {
	writeln!(serial::com1().lock(), "{}", clock::uptime_seconds()).unwrap();
}

fn video_test() {
	vd0().test();
}

fn debug_long(long: u64) {
	debug!("debug_long: {long:016X}");
}

fn video_clear() {
	vdt0().clear();
}

fn read_line(buf: *mut u8) -> isize {
	let line = vdt0().read_line();
	let len = line.len();
	unsafe { ptr::copy_nonoverlapping(line.as_ptr(), buf, len) }
	len as isize
}

fn print_line(ptr: *const u8, len: usize) {
	let slice = unsafe { slice::from_raw_parts(ptr, len) };
	let str = str::from_utf8(slice).expect("Invalid UTF-8 string");
	writeln!(vdt0(), "{str}").expect("VDT0 write error");
}

fn sys_write(_fd: isize, ptr: *const u8, len: usize) {
	let slice = unsafe { slice::from_raw_parts(ptr, len) };
	let str = str::from_utf8(slice).expect("Invalid UTF-8 string");
	write!(serial::com1().lock(), "{str}").expect("COM1 write error");
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

use alloc::{boxed::Box, fmt::format, format, string::String};
use core::{arch::asm, cmp::min, fmt::Write, ptr, slice, str};

use libc::api;
use log::{debug, info, trace, warn};

use crate::{
	arch::amd64::{
		clock,
		vmem::{PageTable, PML4},
	},
	devices::{serial, tty::tty0},
	fs::{
		fs0,
		inode::{Inode, Stat},
		FileDescriptor,
	},
	mem::{frame, page::Page, PhysicalAddress, PAGE_SIZE},
	proc::CPU,
};

#[repr(C)]
#[derive(Debug)]
pub struct RegisterState {
	rdi: u64,
	rsi: u64,
	rbp: u64,
	rsp: u64,
	rbx: u64,
	rdx: u64,
	rcx: u64,
	rax: u64,
	r8: u64,
	r9: u64,
	r10: u64,
	r11: u64,
	r12: u64,
	r13: u64,
	r14: u64,
	r15: u64,
	rip: u64,
}

#[no_mangle]
pub unsafe extern "C" fn syscall_enter(regs: &mut RegisterState) {
	trace!("syscall {}", regs.rax);
	let ret = match regs.rax {
		1 => sys_exit(regs.rdi as isize) as isize,
		2 => sys_brk(regs.rdi),
		3 => sys_open(regs.rdi as *const u8, regs.rsi as usize),
		4 => sys_stat(regs.rdi as usize),
		5 => {
			sys_read(regs.rdi as isize, regs.rsi as *mut u8, regs.rdx as usize)
		}
		6 => sys_write(
			regs.rdi as isize,
			regs.rsi as *const u8,
			regs.rdx as usize,
		),
		7 => sys_readdir(regs.rdi as isize, regs.rsi as *mut api::dirent),
		8 => sys_chdir(regs.rdi as *const u8, regs.rsi as usize),
		9 => sys_fork() as isize,
		10 => sys_fstat(regs.rdi as isize, regs.rsi as *mut api::stat) as isize,
		11 => sys_getcwd(regs.rdi as *mut u8, regs.rsi as usize),
		69 => sys_brk(regs.rdi),
		401 => uptime() as isize,
		403 => debug_long(regs.rdi),
		_ => {
			warn!("unknown syscall: {}", regs.rax);
			-1
		}
	};

	regs.rax = ret as u64;
	trace!("sysret {}", regs.rax);
}

fn uptime() -> u64 {
	clock::uptime_seconds()
}

fn debug_long(long: u64) -> isize {
	debug!("debug_long: {long:016X}");
	0
}

fn sys_read(fd: isize, ptr: *mut u8, len: usize) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };
	let Some(fildes) = task.open_files.get_mut(fd as usize) else {
		return -1;
	};

	kdbg!(fildes.read(ptr, len) as isize)
}

fn sys_chdir(path: *const u8, len: usize) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };
	let path = unsafe {
		let slice = slice::from_raw_parts(path, len);
		str::from_utf8_unchecked(slice)
	};

	if let Some(inode) = fs0().find(&task.cwd, path) {
		task.cwd = inode;
		0
	} else {
		-1
	}
}

fn sys_readdir(fd: isize, ptr: *mut api::dirent) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };
	let Some(fildes) = task.open_files.get_mut(fd as usize) else {
		return -1;
	};
	fildes.readdir(ptr);
	0
}

fn sys_write(_fd: isize, ptr: *const u8, len: usize) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };
	let Some(fd) = task.open_files.get_mut(0) else {
		return -1;
	};
	fd.write(ptr, len) as isize
}

fn sys_open(path: *const u8, len: usize) -> isize {
	trace!("sys_open({path:?}, {len})");

	let slice = unsafe { slice::from_raw_parts(path, len) };
	let fname = str::from_utf8(slice).expect("Invalid UTF-8 string");

	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };

	let Some(inode) = fs0().find(&task.cwd, fname) else {
		return -1;
	};
	let fdesc = FileDescriptor::new(inode);
	task.open_files.push(fdesc);

	return kdbg!(task.open_files.len() as isize - 1);
}

fn sys_stat(fd: usize) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };
	debug!("{:#X?}", task.open_files.get(fd));
	0
}

fn sys_exit(status: isize) -> usize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };

	info!("process {} exited with status: {status}", task.pid);
	breakpoint!();
	0
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

fn sys_fork() -> usize {
	0
}

fn sys_fstat(fildes: isize, buf: *mut api::stat) -> isize {
	let task = CPU::load().current_task();
	let Some(fd) = task.open_files.get_mut(fildes as usize) else {
		return -1;
	};

	let out = unsafe { &mut *buf };
	out.st_mode = fd.inode.mode();
	out.st_size = fd.inode.size() as api::off_t;

	0
}

fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
	let task = CPU::load().current_task();
	let current_inode = &task.cwd;

	fn prepend_parent(child: &Inode) -> Option<String> {
		if let Some(parent) = child.parent() {
			Some(format!(
				"{}/{}",
				prepend_parent(&parent).unwrap_or_default(),
				child.name()
			))
		} else {
			None
		}
	}

	let s = prepend_parent(current_inode).unwrap_or(String::from("/"));
	unsafe { ptr::copy_nonoverlapping(s.as_ptr(), buf, min(len, s.len())) };
	0
}

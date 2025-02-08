use alloc::{format, string::String, vec};
use core::{
	arch::asm, cmp::min, ffi::CStr, mem, mem::size_of, ptr, slice, str,
};

use libc::api;
use log::{debug, info, trace, warn};

use crate::{
	arch::amd64::{
		cli, clock, sti,
		vmem::{
			debug_page_directory, page_table_index, Page, PageTable, Table,
			PML4,
		},
	},
	elf,
	fs::{
		fs0,
		inode::{Inode, Stat},
		FileDescriptor,
	},
	mem::{frame, PhysicalAddress, PAGE_SIZE},
	proc::{Task, CPU},
};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct RegisterState {
	pub rdi: u64,
	pub rsi: u64,
	pub rbp: u64,
	pub rsp: u64,
	rbx: u64,
	rdx: u64,
	pub rcx: u64,
	pub rax: u64,
	r8: u64,
	r9: u64,
	r10: u64,
	r11: u64,
	r12: u64,
	r13: u64,
	r14: u64,
	r15: u64,
	pub rip: u64,
}

#[no_mangle]
pub unsafe extern "C" fn syscall_enter(regs: &mut RegisterState) {
	trace!("syscall {}", regs.rax);
	let ret = match regs.rax {
		1 => sys_exit(regs.rdi as isize, regs),
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
		9 => sys_fork(regs) as isize,
		10 => sys_fstat(regs.rdi as isize, regs.rsi as *mut api::stat) as isize,
		11 => sys_getcwd(regs.rdi as *mut u8, regs.rsi as usize),
		12 => sys_exec(regs.rdi as *mut u8, regs),
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

fn sys_write(fd: isize, ptr: *const u8, len: usize) -> isize {
	trace!("sys_write({:016X?}, {len})", ptr as u64);
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };
	let Some(fd) = task.open_files.get_mut(fd as usize) else {
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

fn sys_exit(status: isize, regs: &mut RegisterState) -> isize {
	let cpu = CPU::load();
	let task = unsafe { &mut *cpu.task };

	info!("process {} exited with status: {status}", task.pid);

	if task.next.is_null() {
		breakpoint!();
	}

	let next_task = unsafe { &mut *task.next };

	// Restore this task's register state.
	debug!(
		"next: {:016X?}, current: {:016X?}",
		&next_task.register_state as *const RegisterState as usize,
		regs as *mut RegisterState as usize
	);
	unsafe { ptr::write(regs as *mut RegisterState, next_task.register_state) };

	cpu.switch_task(next_task);

	// HACK: syscall handler should not replace contents of RAX for every(/any?)
	// syscall.
	next_task.register_state.rax as isize
}

fn sys_brk(addr: u64) -> isize {
	let cpu = CPU::load();
	let task = cpu.current_task();

	let (i_pml4, i_pdp, i_pd) =
		page_table_index(task.register_state.rip as usize);
	let pd = PageTable::<PML4>::current_mut()
		.next_mut(i_pml4)
		.expect("unmapped page that should definitely be mapped")
		.next_mut(i_pdp)
		.expect("unmapped page that should definitely be mapped");

	for i in i_pd..512 {
		if !pd[i].has(Page::PRESENT) {
			let brk = (i * PAGE_SIZE) as isize;

			if addr == 0 {
				trace!("brk: {brk:016X}");
			} else {
				// TODO: Move this whole mess and implement mmap instead.
				let mut frame_allocator = frame::current_mut().lock();
				pd[i] = Page::new(
					frame_allocator.alloc(),
					Page::READ_WRITE | Page::HUGE | Page::USER | Page::PRESENT,
				);
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

fn sys_fork(regs: &mut RegisterState) -> usize {
	let cpu = CPU::load();

	let task = cpu.current_task();

	// Save current task's state.
	task.register_state = regs.clone();
	// Stack pointer in regs is the kernel's, take the user one from GS??
	task.register_state.rsp = cpu.rsp3 as u64;

	let mut new_task = task.fork();
	// Set child pid as return value for parent task.
	task.register_state.rax = new_task.pid;

	// Set CPU's active task.
	// TODO: structure for multitasking
	new_task.next = task as *mut Task;
	cpu.switch_task(new_task);

	// Return 0 to child.
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

fn sys_exec(pathname: *mut u8, regs: &mut RegisterState) -> isize {
	if pathname.is_null() {
		return -1;
	}

	let Ok(path) = unsafe { CStr::from_ptr(pathname as *const i8) }.to_str()
	else {
		return -1;
	};

	let task = CPU::load().current_task();
	let current_inode = &task.cwd;
	let Some(exec_inode) = fs0().find(current_inode, path) else {
		return -1;
	};

	info!("Found path: {}", exec_inode.name());
	if exec_inode.is_dir() {
		return -1;
	}

	// Replace current task with a new page table mapping.
	task.reimage();
	// switch_task() to load the new page table mainly.
	CPU::load().switch_task(task);
	// HACK: switch_task() disables interrupts, but we need them to load the
	//       binary.
	sti();

	// Load binary into task's memory.
	let mut fildes = FileDescriptor::new(exec_inode);
	let sz = fildes.inode.size();
	// HACK: Vec<usize> instead of Vec<u8> is for pointer aligment.
	let mut buf = vec![0usize; sz / usize::BITS as usize];
	fildes.read(buf.as_mut_ptr() as *mut u8, sz);

	elf::load(buf.as_ptr() as *const u8, task);

	// Set up sysret to restore the new register state.
	unsafe { ptr::write(regs as *mut RegisterState, task.register_state) };

	0
}

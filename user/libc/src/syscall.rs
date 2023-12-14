use core::{arch::asm, intrinsics::unreachable};

use crate::api;

#[inline]
fn syscall(number: u64) -> u64 {
	let mut ret;
	unsafe {
		asm!(
			"syscall",
			in("rax") number,
			out("rcx") _,
			out("r11") _,
			lateout("rax") ret
		);
	}
	ret
}

#[inline]
fn syscall1(number: u64, a1: u64) -> u64 {
	let mut ret;
	unsafe {
		asm!(
			"syscall",
			in("rax") number,
			in("rdi") a1,
			out("rcx") _,
			out("r11") _,
			lateout("rax") ret
		);
	}
	ret
}

#[inline]
fn syscall2(number: u64, a1: u64, a2: u64) -> u64 {
	let mut ret;
	unsafe {
		asm!(
		"syscall",
		in("rax") number,
		in("rdi") a1,
		in("rsi") a2,
		out("rcx") _,
		out("r11") _,
		lateout("rax") ret
		);
	}
	ret
}

#[inline]
fn syscall3(number: u64, a1: u64, a2: u64, a3: u64) -> u64 {
	let mut ret;
	unsafe {
		asm!(
			"syscall",
			in("rax") number,
			in("rdi") a1,
			in("rsi") a2,
			in("rdx") a3,
			out("rcx") _,
			out("r11") _,
			lateout("rax") ret
		);
	}
	ret
}

pub fn exit(status: isize) -> ! {
	syscall1(1, status as u64);
	unsafe { unreachable() };
}

pub fn open(path: *const u8, len: usize) -> isize {
	syscall2(3, path as u64, len as u64) as isize
}

pub fn stat(fd: isize) {
	syscall1(4, fd as u64);
}

pub fn write(_fd: u64, buf: *const u8, len: usize) {
	syscall3(6, _fd, buf as u64, len as u64);
}

pub fn chdir(buf: *const u8, len: usize) -> isize {
	syscall2(8, buf as u64, len as u64) as isize
}

pub fn read(_fd: u64, buf: *mut u8, len: usize) -> usize {
	syscall3(5, _fd, buf as u64, len as u64) as usize
}

pub fn readdir(fd: isize, buf: *mut api::dirent) -> isize {
	syscall2(7, fd as u64, buf as u64) as isize
}

pub fn uptime() -> u64 {
	syscall(401)
}

/// Currently brk() is only capable of returning the current break (addr = 0) or
/// incrementing the break by one page (addr != 0).
pub fn brk(addr: usize) -> usize {
	syscall1(2, addr as u64) as usize
}

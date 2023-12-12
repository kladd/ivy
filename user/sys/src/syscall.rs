use alloc::{alloc::alloc, string::String};
use core::{alloc::Layout, arch::asm, intrinsics::unreachable, slice};

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

pub fn debug_long(long: u64) {
	syscall1(403, long);
}

pub fn write(_fd: u64, buf: *const u8, len: usize) {
	syscall3(4, _fd, buf as u64, len as u64);
}

pub fn read(_fd: u64, buf: *mut u8, len: usize) -> usize {
	syscall3(3, _fd, buf as u64, len as u64) as usize
}

/// Currently brk() is only capable of returning the current break (addr = 0) or
/// incrementing the break by one page (addr != 0).
pub fn brk(addr: usize) -> usize {
	syscall1(69, addr as u64) as usize
}

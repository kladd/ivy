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
	syscall1(60, status as u64);
	unsafe { unreachable() };
}

pub fn putc(c: char) {
	syscall1(400, c as u64);
}

pub fn debug_long(long: u64) {
	syscall1(403, long);
}

pub fn video_test() {
	syscall(402);
}

pub fn write(_fd: u64, buf: *const u8, len: usize) {
	syscall3(1, _fd, buf as u64, len as u64);
}

pub fn read_line() -> String {
	let buf = unsafe { alloc(Layout::array::<u8>(80).expect("layout err")) };
	let len = syscall1(405, buf as u64) as usize;
	unsafe { String::from_raw_parts(buf, len, len) }
}

pub fn print_line(str: &str) {
	syscall2(406, str.as_ptr() as u64, str.len() as u64);
}

pub fn video_clear() {
	syscall(404);
}

/// Currently brk() is only capable of returning the current break (addr = 0) or
/// incrementing the break by one page (addr != 0).
pub fn brk(addr: usize) -> usize {
	syscall1(12, addr as u64) as usize
}

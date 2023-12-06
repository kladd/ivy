use core::{arch::asm, intrinsics::unreachable};

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

pub fn exit() -> ! {
	syscall(60);
	unsafe { unreachable() };
}

pub fn putc(c: char) {
	syscall1(400, c as u64);
}

pub fn video_test() {
	syscall(402);
}

pub fn brk(addr: usize) -> usize {
	syscall1(12, addr as u64) as usize
}

use core::{
	fmt::Write,
	sync::atomic::{AtomicU8, Ordering},
};

use log::{info, trace};

use crate::{
	arch::amd64::clock,
	devices::{serial, video::vd0},
};

static LETTER: AtomicU8 = AtomicU8::new('a' as u8);

#[repr(C)]
pub struct RegisterState {
	rcx: u64,
	rdi: u64,
	rax: u64,
}

#[no_mangle]
pub unsafe extern "C" fn syscall_enter(regs: &RegisterState) {
	match regs.rax {
		60 => exit(),
		400 => putc(LETTER.load(Ordering::Acquire) as char),
		401 => uptime(),
		402 => video(),
		_ => trace!("unknown syscall: {}", regs.rax),
	};
}

fn putc(c: char) {
	LETTER.store((c as u8) + 1, Ordering::Release);
	writeln!(serial::com1().lock(), "{c}").unwrap();
}

fn uptime() {
	writeln!(serial::com1().lock(), "{}", clock::uptime_seconds()).unwrap();
}

fn video() {
	vd0().test();
}

fn exit() {
	breakpoint!();
}

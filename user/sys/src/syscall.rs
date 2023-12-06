use core::{arch::asm, intrinsics::unreachable};

#[inline]
fn syscall(number: usize) {
	unsafe {
		asm!("syscall", in("rax") number);
	}
}

#[no_mangle]
pub fn exit() -> ! {
	syscall(60);
	unsafe { unreachable() };
}

#[no_mangle]
pub fn video_test() {
	syscall(402);
}

use core::{arch::asm, intrinsics::unreachable};

fn syscall(number: usize) {
	unsafe {
		asm!("syscall", in("rax") number);
	}
}

pub fn exit() -> ! {
	syscall(60);
	unsafe { unreachable() };
}

pub fn video_test() {
	syscall(402);
}

use core::{arch::global_asm, panic::PanicInfo};

use crate::syscall::exit;

global_asm! {
	".global _start",
	"_start:",
	"call main",
	"mov rdi, rax",
	"mov eax, 60",
	"syscall"
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	exit();
}

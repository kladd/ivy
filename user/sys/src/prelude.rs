use core::panic::PanicInfo;

use crate::{malloc::Allocator, sync::SpinLock, syscall::exit};

extern "C" {
	fn main() -> isize;
}

#[global_allocator]
static GLOBAL_ALLOC: SpinLock<Allocator> = SpinLock::new(Allocator::new());

#[export_name = "_start"]
unsafe fn _start() {
	GLOBAL_ALLOC.lock().init();
	exit(main())
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	exit(101);
}

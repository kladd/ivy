use core::panic::PanicInfo;

use crate::syscall::exit;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	exit();
}

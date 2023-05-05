#[allow(unused_macros)]
macro_rules! kdbg {
	() => {
		::log::debug!("").unwrap();
	};
	($val:expr $(,)?) => {
		match $val {
			v => {
				::log::debug!("{} = {v:#?}", stringify!($val));
				v
			}
		}
	};
	($($val:expr),+ $(,)?) => {
		($($crate::kdbg!($val)),+,)
	};
}

macro_rules! dump_register {
	($arg:tt) => {
		let mut tmp: u32 = 0;
		unsafe { core::arch::asm!(concat!("mov {}, ", $arg), out(reg) tmp); }
		kprintf!("{}: 0x{:08X} 0b{:032b}", $arg, tmp, tmp);
	};
}

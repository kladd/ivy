#[allow(unused_macros)]
#[macro_export]
macro_rules! kdbg {
	() => {
		::log::debug!("").unwrap();
	};
	($val:expr $(,)?) => {
		match $val {
			v => {
				::log::debug!("{} = {v:#X?}", stringify!($val));
				v
			}
		}
	};
	($($val:expr),+ $(,)?) => {
		($($crate::kdbg!($val)),+,)
	};
}

macro_rules! breakpoint {
	() => {
		unsafe { ::core::arch::asm!("int 3") }
	};
}

#[allow(unused_macros)]
macro_rules! kdbg {
	() => {
		writeln!(crate::serial::COM1, "[{}:{}]", file!(), line!()).unwrap();
	};
	($val:expr $(,)?) => {
		match $val {
			v => {
				writeln!(
					crate::serial::COM1,
					"[{}:{}] {} = {:#?}",
					file!(),
					line!(),
					stringify!($val),
					&v
				).unwrap();
				v
			}
		}
	};
	($($val:expr),+ $(,)?) => {
		($($crate::kdbg!($val)),+,)
	};
}

#[allow(unused_macros)]
macro_rules! kprintf {
	($($arg:tt)*) => {
		writeln!(crate::serial::COM1, "[{}:{}] {}", file!(), line!(), format_args!($($arg)*)).unwrap();
	};
}

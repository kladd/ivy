use alloc::vec::Vec;
use core::{
	fmt,
	fmt::{Debug, Display, Formatter, Write},
	ops::Deref,
};

pub struct String {
	vec: Vec<u8>,
}

impl String {
	pub fn new(capacity: usize) -> Self {
		Self {
			vec: Vec::with_capacity(capacity),
		}
	}

	pub fn from_ascii_own(vec: Vec<u8>) -> Self {
		Self { vec }
	}

	pub fn pop(&mut self) -> Option<u8> {
		self.vec.pop()
	}
}

impl Deref for String {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		unsafe { core::str::from_utf8_unchecked(&self.vec) }
	}
}

impl AsRef<str> for String {
	fn as_ref(&self) -> &str {
		self
	}
}

impl Display for String {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self.deref(), f)
	}
}

impl Debug for String {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self.deref(), f)
	}
}

impl Write for String {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for byte in s.bytes() {
			self.vec.push(byte);
		}

		Ok(())
	}
}

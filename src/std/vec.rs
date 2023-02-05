use core::{fmt::Write, mem::size_of, ops::Deref, slice};

use crate::std::alloc::kmalloc;

pub struct Vec<T> {
	start: *mut T,
	capacity: usize,
	len: usize,
}

impl<T> Vec<T> {
	pub fn new(capacity: usize) -> Self {
		Self {
			start: kdbg!(kmalloc(capacity * size_of::<T>()) as *mut T),
			capacity,
			len: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn push(&mut self, item: T) {
		self.put(self.len, item);
		self.len += 1;
	}

	pub fn put(&mut self, i: usize, item: T) {
		if i > self.capacity {
			panic!("index out of range: {}", i);
		}
		unsafe { *self.start.offset(i as isize) = item }
	}

	pub fn get(&self, i: usize) -> &T {
		if i > self.capacity {
			panic!("index out of range: {}", i);
		}
		unsafe { &*self.start.offset(i as isize) }
	}
}

impl<T> Deref for Vec<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		unsafe { slice::from_raw_parts(self.start, self.len) }
	}
}

use core::{fmt::Write, mem::size_of, ops::Deref, ptr, slice};

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

	pub unsafe fn copy_from_ptr(src: *const T, count: usize) -> Self {
		let vec = Self {
			start: kdbg!(kmalloc(count * size_of::<T>()) as *mut T),
			capacity: count,
			len: count,
		};

		ptr::copy(src, vec.start, count);

		vec
	}
}

impl<T> Deref for Vec<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		unsafe { slice::from_raw_parts(self.start, self.len) }
	}
}

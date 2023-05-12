use alloc::boxed::Box;
use core::{
	ptr::null_mut,
	sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

pub struct Global<T: Send + Sync> {
	initialized: AtomicBool,
	value: AtomicPtr<T>,
}

impl<T: Send + Sync> Global<T> {
	pub const fn new() -> Self {
		Self {
			initialized: AtomicBool::new(false),
			value: AtomicPtr::new(null_mut()),
		}
	}

	pub fn init(&self, val: T) {
		if self
			.initialized
			.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
			.unwrap()
		{
			self.value
				.store(Box::leak(Box::new(val)), Ordering::Relaxed);
		}
	}

	pub fn instance_mut(&self) -> &mut T {
		if !self.initialized.load(Ordering::Acquire) {
			todo!()
		} else {
			unsafe { &mut *self.value.load(Ordering::Relaxed) }
		}
	}
}

impl<T: Sync + Send> Drop for Global<T> {
	fn drop(&mut self) {}
}

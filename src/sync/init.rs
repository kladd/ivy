use core::{
	cell::UnsafeCell,
	sync::atomic::{AtomicBool, Ordering},
};

pub struct InitOnce<T: Sync> {
	done: AtomicBool,
	data: UnsafeCell<Option<T>>,
}

impl<T: Sync> InitOnce<T> {
	pub const fn new() -> Self {
		Self {
			done: AtomicBool::new(false),
			data: UnsafeCell::new(None),
		}
	}

	pub fn get_or_init<F>(&self, f: F) -> &T
	where
		F: FnOnce() -> T,
	{
		if self
			.done
			.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
			.is_ok()
		{
			unsafe { *self.data.get() = Some(f()) };
		}

		unsafe { (*self.data.get()).as_ref().unwrap() }
	}
}

unsafe impl<T: Sync> Sync for InitOnce<T> {}

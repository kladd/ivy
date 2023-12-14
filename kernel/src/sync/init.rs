use alloc::boxed::Box;
use core::{
	cell::UnsafeCell,
	ptr,
	sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

pub struct InitOnce<T: Sync> {
	done: AtomicBool,
	data: UnsafeCell<Option<T>>,
}

pub struct StaticPtr<T>(AtomicPtr<T>);

impl<T> StaticPtr<T> {
	pub const fn new() -> Self {
		Self(AtomicPtr::new(ptr::null_mut()))
	}

	pub fn init(&self, value: T) {
		self.0
			.compare_exchange(
				ptr::null_mut(),
				Box::into_raw(Box::new(value)),
				Ordering::SeqCst,
				Ordering::Relaxed,
			)
			.expect("StaticPtr already initialized");
	}

	pub fn get(&self) -> &mut T {
		let val = self.0.load(Ordering::Acquire);
		if val.is_null() {
			panic!("Accessed uninitialized static ptr");
		}
		unsafe { &mut *val }
	}
}

// TODO: make T Sync.
unsafe impl<T> Sync for StaticPtr<T> {}

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

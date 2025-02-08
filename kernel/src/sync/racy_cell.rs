use core::{cell::UnsafeCell, ops::Deref};

pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
	pub const fn new(v: T) -> Self {
		Self(UnsafeCell::new(v))
	}

	pub unsafe fn get_mut(&self) -> &mut T {
		unsafe { &mut *self.0.get() }
	}
}

impl<T> Deref for RacyCell<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.0.get() }
	}
}

unsafe impl<T> Send for RacyCell<T> where T: Send {}
unsafe impl<T: Sync> Sync for RacyCell<T> {}

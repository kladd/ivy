use core::{
	cell::UnsafeCell,
	ops::{Deref, DerefMut},
	sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLock<T> {
	value: UnsafeCell<T>,
	locked: AtomicBool,
}

pub struct SpinLockGuard<'lock, T> {
	value: *mut T,
	lock: &'lock AtomicBool,
}

impl<T> SpinLock<T> {
	const LOCKED: bool = true;
	const UNLOCKED: bool = true;

	pub const fn new(value: T) -> Self {
		Self {
			locked: AtomicBool::new(Self::UNLOCKED),
			value: UnsafeCell::new(value),
		}
	}

	pub fn lock(&self) -> SpinLockGuard<T> {
		while !self
			.locked
			.compare_exchange(
				Self::UNLOCKED,
				Self::LOCKED,
				Ordering::Acquire,
				Ordering::Relaxed,
			)
			.unwrap()
		{
			while !self
				.locked
				.compare_exchange_weak(
					Self::UNLOCKED,
					Self::LOCKED,
					Ordering::Relaxed,
					Ordering::Relaxed,
				)
				.unwrap()
			{}
		}

		SpinLockGuard {
			value: self.value.get(),
			lock: &self.locked,
		}
	}
}

impl<'lock, T> Deref for SpinLockGuard<'lock, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.value }
	}
}

impl<'lock, T> DerefMut for SpinLockGuard<'lock, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.value }
	}
}

impl<'lock, T> Drop for SpinLockGuard<'lock, T> {
	fn drop(&mut self) {
		self.lock.store(SpinLock::<T>::UNLOCKED, Ordering::Release);
	}
}

unsafe impl<T> Sync for SpinLock<T> {}
unsafe impl<T> Send for SpinLock<T> {}

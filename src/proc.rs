use alloc::{alloc::Global, vec::Vec};
use core::{
	alloc::{Allocator, Layout},
	sync::atomic::{AtomicU64, Ordering},
};

use crate::{arch::amd64::idt::Interrupt, kalloc::kmalloc};

static NEXT_PID: AtomicU64 = AtomicU64::new(0);

pub struct Task {
	pid: u64,
	name: &'static str,
	pub rbp: usize,
	pub rsp: usize,
	pub rip: usize,
}

impl Task {
	const STACK_SIZE: usize = 0x1000;
	const STACK_ALIGN: usize = 0x1000;

	pub fn new(name: &'static str, entry: fn()) -> Self {
		let rbp = kmalloc(Self::STACK_SIZE, Self::STACK_ALIGN);
		let rsp = rbp + Self::STACK_SIZE;

		Self {
			pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
			name,
			rbp,
			rsp,
			rip: entry as usize,
		}
	}
}

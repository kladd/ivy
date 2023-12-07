#![no_std]
#![feature(core_intrinsics)]

extern crate alloc;

pub mod malloc;
pub mod prelude;
pub mod sync;
pub mod syscall;

pub const PAGE_SIZE: usize = 0x200000;

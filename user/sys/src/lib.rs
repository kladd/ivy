#![no_std]
#![feature(core_intrinsics)]

pub mod prelude;
pub mod syscall;

pub const PAGE_SIZE: usize = 0x200000;

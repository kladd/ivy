#![no_std]
#![feature(core_intrinsics)]

extern crate alloc;

pub mod api;
pub mod dirent;
pub mod fcntl;
mod malloc;
#[cfg(not(feature = "kernel"))]
pub mod prelude;
mod sync;
mod syscall;
pub mod unistd;

pub const PAGE_SIZE: usize = 0x200000;

#![no_std]

extern crate alloc;

pub mod api;
pub mod dirent;
pub mod fcntl;
pub mod malloc;
#[cfg(not(feature = "kernel"))]
pub mod prelude;
mod stat;
pub mod sync;
pub mod syscall;
pub mod unistd;

pub const PAGE_SIZE: usize = 0x200000;

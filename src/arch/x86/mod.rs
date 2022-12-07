use core::{arch::global_asm, include_str};

#[link_section = ".multiboot"]
static _MULTIBOOT_MAGIC: i32 = 0x1BADB002;
#[link_section = ".multiboot"]
// ALIGN | MEM_INFO
static _MULTIBOOT_FLAGS: i32 = 3;
#[link_section = ".multiboot"]
static _MULTIBOOT_CHK: i32 = -(_MULTIBOOT_MAGIC + _MULTIBOOT_FLAGS);

global_asm!(include_str!("boot.asm"));

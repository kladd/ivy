use core::{
	fmt::Write,
	ptr::{read_volatile, write_volatile},
};

use crate::{
	dev::character::{Keycode, WriteCharacter},
	sync::{
		init::InitOnce,
		spin_lock::{SpinLock, SpinLockGuard},
	},
};

#[cfg(not(feature = "hardware"))]
const UART0_BASE: usize = 0x09000000;
#[cfg(feature = "hardware")]
const UART0_BASE: usize = 0xFE20_1000;

const UART0_DR: usize = UART0_BASE + 0x00;
const UART0_FR: usize = UART0_BASE + 0x18;
const UART0_IBRD: usize = UART0_BASE + 0x24;
const UART0_FBRD: usize = UART0_BASE + 0x28;
const UART0_LCRH: usize = UART0_BASE + 0x2C;
const UART0_CR: usize = UART0_BASE + 0x30;
const UART0_IMSC: usize = UART0_BASE + 0x38;
const UART0_DMACR: usize = UART0_BASE + 0x48;

const CR_UART_EN: u32 = 1 << 0;
const CR_TX_EN: u32 = 1 << 8;
const CR_RX_EN: u32 = 1 << 9;

const FR_BUSY: u32 = 1 << 3;

const LCRH_F_EN: u32 = 1 << 4;
const LCRH_WLEN8: u32 = (0x3 << 5);

const UART_CLK: u32 = 24000000;
const UART_BAUD: u32 = 115200;

static UART0: InitOnce<SpinLock<UartHandle>> = InitOnce::new();

pub fn uart() -> Uart {
	Uart {
		inner: UART0.get_or_init(|| SpinLock::new(UartHandle::new())),
	}
}

fn init() {
	disable_uart();
	await_tx();

	flush_fifo();
	set_115200_baud();
	set_8n1_line_control();
	mask_interrupts();
	disable_dma();

	enable_uart();
}

fn disable_uart() {
	unsafe {
		let cr = read_volatile(UART0_CR as *mut u32);
		write_volatile(UART0_CR as *mut u32, cr & CR_UART_EN);
	}
}

fn mask_interrupts() {
	unsafe { write_volatile(UART0_IMSC as *mut u32, 0x7FF) };
}

fn set_115200_baud() {
	let div = 4 * UART_CLK / UART_BAUD;
	unsafe { write_volatile(UART0_FBRD as *mut u32, div & 0x3F) };
	unsafe { write_volatile(UART0_IBRD as *mut u32, (div >> 6) & 0xFFFF) };
}

fn set_8n1_line_control() {
	unsafe { write_volatile(UART0_LCRH as *mut u32, LCRH_F_EN | LCRH_WLEN8) };
}

fn flush_fifo() {
	unsafe {
		let lcr = read_volatile(UART0_LCRH as *mut u32);
		write_volatile(UART0_LCRH as *mut u32, lcr & !LCRH_F_EN);
	}
}

fn enable_uart() {
	unsafe {
		write_volatile(UART0_CR as *mut u32, CR_UART_EN | CR_TX_EN | CR_RX_EN)
	};
}

fn disable_dma() {
	unsafe { write_volatile(UART0_DMACR as *mut u32, 0) };
}

fn await_tx() {
	unsafe { while (read_volatile(UART0_FR as *mut u32) & FR_BUSY) != 0 {} }
}

fn write_byte(byte: u8) {
	await_tx();
	unsafe {
		write_volatile(UART0_DR as *mut u32, byte as u32);
	}
}

pub struct Uart {
	inner: &'static SpinLock<UartHandle>,
}

impl Uart {
	pub fn lock(&self) -> UartLockGuard<'static> {
		UartLockGuard {
			_inner: self.inner.lock(),
		}
	}
}

struct UartHandle;

impl UartHandle {
	fn new() -> Self {
		init();
		UartHandle
	}
}

impl Drop for UartHandle {
	fn drop(&mut self) {
		disable_uart();
	}
}

pub struct UartLockGuard<'a> {
	_inner: SpinLockGuard<'a, UartHandle>,
}

impl WriteCharacter for UartLockGuard<'_> {
	fn putc(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Char(c) => write_byte(c as u8),
			Keycode::Backspace => self.write_str("\x08 \x08").unwrap(),
			_ => { /* TODO: Nak, Formfeed, etc. */ }
		}
	}
}

impl Write for UartLockGuard<'_> {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		for b in s.bytes() {
			write_byte(b);
		}
		Ok(())
	}
}

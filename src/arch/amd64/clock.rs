use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use log::trace;

use crate::arch::amd64::{
	idt::{register_handler, Interrupt},
	inb, outb,
};

const PIT_FREQ: u32 = 18;

const RTC_CMD: u16 = 0x70;
const RTC_DAT: u16 = 0x71;

const RTC_SECOND: u8 = 0x00;
const RTC_MINUTE: u8 = 0x02;
const RTC_HOUR: u8 = 0x04;
const RTC_DAY: u8 = 0x07;
const RTC_MONTH: u8 = 0x08;
const RTC_YEAR: u8 = 0x09;
const RTC_CENTURY: u8 = 0x32;

static CLOCK: AtomicU64 = AtomicU64::new(0);

pub fn init_clock() {
	// Set interval interrupt handler.
	register_handler(32, handle_interval_timer);
}

pub fn uptime_seconds() -> u64 {
	// 18.222 (repeating of course), so not accurate here really
	CLOCK.load(Ordering::Relaxed) / 18
}

pub extern "x86-interrupt" fn handle_interval_timer(_: Interrupt) {
	// Send EOI
	outb(0x20, 0x20);
	CLOCK.fetch_add(1, Ordering::Relaxed);
}

fn rtc(register: u8) -> u8 {
	outb(RTC_CMD, register);
	inb(RTC_DAT)
}

fn bcd_rtc(register: u8) -> u8 {
	let x = rtc(register);
	(x & 0x0F) + ((x / 16) * 10)
}

pub fn year() -> u16 {
	(bcd_rtc(RTC_CENTURY) as u16 * 100) + bcd_rtc(RTC_YEAR) as u16
}

pub fn month() -> u8 {
	bcd_rtc(RTC_MONTH)
}

pub fn day() -> u8 {
	bcd_rtc(RTC_DAY)
}

pub fn hour() -> u8 {
	let h = rtc(RTC_HOUR);
	((h & 0x0F) + (((h & 0x70) / 16) * 10)) | (h & 0x80)
}

pub fn minute() -> u8 {
	bcd_rtc(RTC_MINUTE)
}

pub fn second() -> u8 {
	bcd_rtc(RTC_SECOND)
}

use core::fmt::Write;

use log::{Level, Metadata, Record};

use crate::serial;

pub struct KernelLogger;

const COLOR_DEFAULT: &'static str = "\x1b[0m";
const COLOR_GREY: &'static str = "\x1b[90m";
const COLOR_RED: &'static str = "\x1b[91m";
const COLOR_YELLOW: &'static str = "\x1b[93m";

impl log::Log for KernelLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= log::STATIC_MAX_LEVEL
	}

	fn log(&self, record: &Record) {
		if self.enabled(record.metadata()) {
			writeln!(
				serial::COM1,
				"{}{:>5}{} [{}:{}]: {}",
				Self::start_color(record.metadata()),
				record.level(),
				COLOR_DEFAULT,
				record.file().unwrap(),
				record.line().unwrap(),
				record.args()
			)
			.unwrap();
		}
	}

	fn flush(&self) {}
}

impl KernelLogger {
	fn start_color(md: &Metadata) -> &'static str {
		match md.level() {
			Level::Debug | Level::Trace => COLOR_GREY,
			Level::Warn => COLOR_YELLOW,
			Level::Error => COLOR_RED,
			Level::Info => COLOR_DEFAULT,
		}
	}
}

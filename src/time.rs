use core::{
	fmt::{Display, Formatter},
	mem,
};

use crate::arch::x86::clock;

pub struct DateTime {
	date: Date,
	time: Time,
}

struct Date {
	day: u8,
	month: u8,
	year: u16,
}

struct Time {
	hour: u8,
	minute: u8,
	second: u8,
}

#[derive(Debug)]
pub enum Day {
	Sun,
	Mon,
	Tue,
	Wed,
	Thu,
	Fri,
	Sat,
}

#[derive(Debug)]
pub enum Month {
	Jan = 1,
	Feb,
	Mar,
	Apr,
	May,
	Jun,
	Jul,
	Aug,
	Sep,
	Oct,
	Nov,
	Dec,
}

impl DateTime {
	pub fn now() -> Self {
		Self {
			date: Date {
				day: clock::day(),
				month: clock::month(),
				year: clock::year(),
			},
			time: Time {
				hour: clock::hour(),
				minute: clock::minute(),
				second: clock::second(),
			},
		}
	}

	pub fn year(&self) -> u16 {
		self.date.year
	}

	pub fn month(&self) -> u8 {
		self.date.month
	}

	pub fn day(&self) -> u8 {
		self.date.day
	}

	pub fn hour(&self) -> u8 {
		self.time.hour
	}

	pub fn minute(&self) -> u8 {
		self.time.minute
	}

	pub fn second(&self) -> u8 {
		self.time.second
	}

	// Keith--Craver method-ish. Sunday is 0.
	fn day_of_week(&self) -> Day {
		let mut y = self.date.year;
		let mut d = self.date.day as u16;
		let m = self.date.month as u16;

		d += if self.date.month < 3 {
			y -= 1;
			y
		} else {
			y - 2
		};

		Day::from(((23 * m / 9 + d + 4 + y / 4 - y / 100 + y / 400) % 7) as u8)
	}
}

impl Display for DateTime {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.write_fmt(format_args!(
			"{:?} {:?} {:2} {:02}:{:02}:{:02} UTC {}",
			self.day_of_week(),
			Month::from(self.date.month),
			self.date.day,
			self.time.hour,
			self.time.minute,
			self.time.second,
			self.date.year
		))
	}
}

impl From<u8> for Day {
	fn from(value: u8) -> Self {
		assert!(value <= Self::Sat as u8);
		unsafe { mem::transmute::<u8, Self>(value) }
	}
}

impl From<u8> for Month {
	fn from(value: u8) -> Self {
		assert!(value <= Self::Dec as u8);
		unsafe { mem::transmute::<u8, Self>(value) }
	}
}

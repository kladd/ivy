use crate::{
	arch::amd64::{
		idt::{register_handler, Interrupt},
		inb, outb,
	},
	devices::character::{Keycode, ReadCharacter},
};

const NUL: char = 0 as char;

const ASCII_NO_MOD: [char; 89] = [
	NUL, NUL, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', NUL,
	NUL, 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', NUL,
	'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', NUL, '\\',
	'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', NUL, '*', NUL, ' ', NUL,
	NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
	NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
];

const ASCII_MOD_SHIFT: [char; 89] = [
	NUL, NUL, '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', NUL,
	NUL, 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', NUL, NUL,
	'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~', NUL, '|', 'Z',
	'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?', NUL, '*', NUL, ' ', NUL, NUL,
	NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
	NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL, NUL,
];

const MOD_ALT: u8 = 1;
const MOD_CTRL: u8 = 2;
const MOD_SHIFT: u8 = 4;

const I8042_STATUS_PORT: u16 = 0x64;
const I8042_BUFFER_PORT: u16 = 0x60;
const I8042_KEY_DEPRESSED: u8 = 0x80;

const STATUS_DATA_AVAILABLE: u8 = 0x1;

pub const BUFFER_SIZE: usize = 16;

static mut MODS: u8 = 0;

pub static mut KBD: Keyboard<BUFFER_SIZE> = Keyboard {
	index: 0,
	buffer: [Keycode::Null; BUFFER_SIZE],
};

#[derive(Debug)]
pub struct Keyboard<const N: usize> {
	index: usize,
	buffer: [Keycode; N],
}

impl<const N: usize> ReadCharacter for Keyboard<N> {
	fn getc(&mut self) -> Option<Keycode> {
		if self.index == 0 {
			None
		} else {
			self.index -= 1;
			Some(self.buffer[self.index])
		}
	}
}

impl<const N: usize> Keyboard<N> {
	fn putc(&mut self, c: Keycode) {
		let next = (self.index + 1) % N;
		if next != 0 {
			self.buffer[self.index] = c;
			self.index = next;
		} else {
			panic!("Keyboard buffer full!");
		}
	}
}

pub fn init_keyboard() {
	register_handler(0x21, irq_handler);
}

fn keyboard_has_data() -> bool {
	(inb(I8042_STATUS_PORT) & STATUS_DATA_AVAILABLE) != 0
}

fn keyboard_read_scan_code() -> u8 {
	inb(I8042_BUFFER_PORT)
}

fn is_key_down(scan_code: u8) -> bool {
	scan_code & I8042_KEY_DEPRESSED == 0
}

fn mod_shift() -> bool {
	unsafe { MODS & MOD_SHIFT != 0 }
}

fn mod_ctrl() -> bool {
	unsafe { MODS & MOD_CTRL != 0 }
}

extern "x86-interrupt" fn irq_handler(_: Interrupt) {
	while keyboard_has_data() {
		unsafe {
			match keyboard_read_scan_code() {
				0x38 => MODS |= MOD_ALT,
				0x1D => MODS |= MOD_CTRL,
				0x2A => MODS |= MOD_SHIFT,

				0xB8 => MODS &= !MOD_ALT,
				0x9D => MODS &= !MOD_CTRL,
				0xAA => MODS &= !MOD_SHIFT,

				0x16 if mod_ctrl() => KBD.putc(Keycode::Nak),
				0x1E if mod_ctrl() => KBD.putc(Keycode::StartOfHeading),
				0x25 if mod_ctrl() => KBD.putc(Keycode::VerticalTab),
				0x26 if mod_ctrl() => KBD.putc(Keycode::FormFeed),

				0x0E => KBD.putc(Keycode::Backspace),
				0x1C => KBD.putc(Keycode::Newline),

				scan_code if is_key_down(scan_code) => {
					let c = if mod_shift() {
						ASCII_MOD_SHIFT[scan_code as usize]
					} else {
						ASCII_NO_MOD[scan_code as usize]
					};

					if c != NUL {
						KBD.putc(Keycode::Char(c))
					}
				}

				_ => continue,
			}
		}
	}

	// EOI TODO: like make this not explicit.
	outb(0x20, 0x20);
}
